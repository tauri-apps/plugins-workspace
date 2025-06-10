// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Store secrets and keys using the [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs) encrypted database and secure runtime.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use std::{
    collections::HashMap,
    fmt,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use crypto::keys::bip39;
use iota_stronghold::{
    procedures::{
        BIP39Generate, BIP39Recover, Curve, Ed25519Sign, KeyType as StrongholdKeyType,
        MnemonicLanguage, PublicKey, Slip10Derive, Slip10DeriveInput, Slip10Generate,
        StrongholdProcedure,
    },
    Client, Location,
};
use serde::{de::Visitor, Deserialize, Deserializer};
use stronghold::{Error, Result, Stronghold};
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State,
};
use zeroize::{Zeroize, Zeroizing};

#[cfg(feature = "kdf")]
pub mod kdf;

pub mod stronghold;

type PasswordHashFn = dyn Fn(&str) -> Vec<u8> + Send + Sync;

#[derive(Default)]
struct StrongholdCollection(Arc<Mutex<HashMap<PathBuf, Stronghold>>>);

struct PasswordHashFunction(Box<PasswordHashFn>);

#[derive(Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(untagged)]
enum BytesDto {
    Text(String),
    Raw(Vec<u8>),
}

impl AsRef<[u8]> for BytesDto {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Text(t) => t.as_ref(),
            Self::Raw(b) => b.as_ref(),
        }
    }
}

impl From<BytesDto> for Vec<u8> {
    fn from(v: BytesDto) -> Self {
        match v {
            BytesDto::Text(t) => t.as_bytes().to_vec(),
            BytesDto::Raw(b) => b,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
enum LocationDto {
    Generic { vault: BytesDto, record: BytesDto },
    Counter { vault: BytesDto, counter: usize },
}

impl From<LocationDto> for Location {
    fn from(dto: LocationDto) -> Location {
        match dto {
            LocationDto::Generic { vault, record } => Location::generic(vault, record),
            LocationDto::Counter { vault, counter } => Location::counter(vault, counter),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
#[allow(clippy::upper_case_acronyms)]
enum Slip10DeriveInputDto {
    Seed(LocationDto),
    Key(LocationDto),
}

impl From<Slip10DeriveInputDto> for Slip10DeriveInput {
    fn from(dto: Slip10DeriveInputDto) -> Slip10DeriveInput {
        match dto {
            Slip10DeriveInputDto::Seed(location) => Slip10DeriveInput::Seed(location.into()),
            Slip10DeriveInputDto::Key(location) => Slip10DeriveInput::Key(location.into()),
        }
    }
}

pub enum KeyType {
    Ed25519,
    X25519,
}

impl From<KeyType> for StrongholdKeyType {
    fn from(ty: KeyType) -> StrongholdKeyType {
        match ty {
            KeyType::Ed25519 => StrongholdKeyType::Ed25519,
            KeyType::X25519 => StrongholdKeyType::X25519,
        }
    }
}

impl<'de> Deserialize<'de> for KeyType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyTypeVisitor;

        impl Visitor<'_> for KeyTypeVisitor {
            type Value = KeyType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("ed25519 or x25519")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value.to_lowercase().as_str() {
                    "ed25519" => Ok(KeyType::Ed25519),
                    "x25519" => Ok(KeyType::X25519),
                    _ => Err(serde::de::Error::custom("unknown key type")),
                }
            }
        }

        deserializer.deserialize_str(KeyTypeVisitor)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
#[allow(clippy::upper_case_acronyms)]
enum ProcedureDto {
    SLIP10Generate {
        output: LocationDto,
        #[serde(rename = "sizeBytes")]
        size_bytes: Option<usize>,
    },
    SLIP10Derive {
        chain: Vec<u32>,
        input: Slip10DeriveInputDto,
        output: LocationDto,
    },
    BIP39Recover {
        mnemonic: String,
        passphrase: Option<String>,
        output: LocationDto,
    },
    BIP39Generate {
        passphrase: Option<String>,
        output: LocationDto,
    },
    PublicKey {
        #[serde(rename = "type")]
        ty: KeyType,
        #[serde(rename = "privateKey")]
        private_key: LocationDto,
    },
    Ed25519Sign {
        #[serde(rename = "privateKey")]
        private_key: LocationDto,
        msg: String,
    },
}

impl From<ProcedureDto> for StrongholdProcedure {
    fn from(dto: ProcedureDto) -> StrongholdProcedure {
        match dto {
            ProcedureDto::SLIP10Generate { output, size_bytes } => {
                StrongholdProcedure::Slip10Generate(Slip10Generate {
                    output: output.into(),
                    size_bytes,
                })
            }
            ProcedureDto::SLIP10Derive {
                chain,
                input,
                output,
            } => StrongholdProcedure::Slip10Derive(Slip10Derive {
                curve: Curve::Ed25519,
                chain,
                input: input.into(),
                output: output.into(),
            }),
            ProcedureDto::BIP39Recover {
                mnemonic,
                passphrase,
                output,
            } => StrongholdProcedure::BIP39Recover(BIP39Recover {
                mnemonic: bip39::Mnemonic::from(mnemonic),
                passphrase: bip39::Passphrase::from(passphrase.unwrap_or_default()),
                output: output.into(),
            }),
            ProcedureDto::BIP39Generate { passphrase, output } => {
                StrongholdProcedure::BIP39Generate(BIP39Generate {
                    passphrase: bip39::Passphrase::from(passphrase.unwrap_or_default()),
                    output: output.into(),
                    language: MnemonicLanguage::English,
                })
            }
            ProcedureDto::PublicKey { ty, private_key } => {
                StrongholdProcedure::PublicKey(PublicKey {
                    ty: ty.into(),
                    private_key: private_key.into(),
                })
            }
            ProcedureDto::Ed25519Sign { private_key, msg } => {
                StrongholdProcedure::Ed25519Sign(Ed25519Sign {
                    private_key: private_key.into(),
                    msg: msg.as_bytes().to_vec(),
                })
            }
        }
    }
}

#[tauri::command]
async fn initialize(
    collection: State<'_, StrongholdCollection>,
    hash_function: State<'_, PasswordHashFunction>,
    snapshot_path: PathBuf,
    mut password: String,
) -> Result<()> {
    let hash = (hash_function.0)(&password);
    password.zeroize();
    let stronghold = Stronghold::new(snapshot_path.clone(), hash)?;

    collection
        .0
        .lock()
        .unwrap()
        .insert(snapshot_path, stronghold);

    Ok(())
}

#[tauri::command]
async fn destroy(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
) -> Result<()> {
    let mut collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.remove(&snapshot_path) {
        if let Err(e) = stronghold.save() {
            collection.insert(snapshot_path, stronghold);
            return Err(e);
        }
    }
    Ok(())
}

#[tauri::command]
async fn save(collection: State<'_, StrongholdCollection>, snapshot_path: PathBuf) -> Result<()> {
    let collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.get(&snapshot_path) {
        stronghold.save()?;
    }
    Ok(())
}

#[tauri::command]
async fn create_client(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
) -> Result<()> {
    let stronghold = get_stronghold(collection, snapshot_path)?;
    stronghold.create_client(client)?;
    Ok(())
}

#[tauri::command]
async fn load_client(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
) -> Result<()> {
    let stronghold = get_stronghold(collection, snapshot_path)?;
    stronghold.load_client(client)?;
    Ok(())
}

#[tauri::command]
async fn get_store_record(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    key: String,
) -> Result<Option<Vec<u8>>> {
    let client = get_client(collection, snapshot_path, client)?;
    client.store().get(key.as_ref()).map_err(Into::into)
}

#[tauri::command]
async fn save_store_record(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    key: String,
    value: Vec<u8>,
    lifetime: Option<Duration>,
) -> Result<Option<Vec<u8>>> {
    let client = get_client(collection, snapshot_path, client)?;
    client
        .store()
        .insert(key.as_bytes().to_vec(), value, lifetime)
        .map_err(Into::into)
}

#[tauri::command]
async fn remove_store_record(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    key: String,
) -> Result<Option<Vec<u8>>> {
    let client = get_client(collection, snapshot_path, client)?;
    client.store().delete(key.as_ref()).map_err(Into::into)
}

#[tauri::command]
async fn save_secret(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    vault: BytesDto,
    record_path: BytesDto,
    secret: Vec<u8>,
) -> Result<()> {
    let client = get_client(collection, snapshot_path, client)?;
    client
        .vault(&vault)
        .write_secret(
            Location::generic(vault, record_path),
            Zeroizing::new(secret),
        )
        .map_err(Into::into)
}

#[tauri::command]
async fn remove_secret(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    vault: BytesDto,
    record_path: BytesDto,
) -> Result<()> {
    let client = get_client(collection, snapshot_path, client)?;
    client
        .vault(vault)
        .delete_secret(record_path)
        .map(|_| ())
        .map_err(Into::into)
}

#[tauri::command]
async fn execute_procedure(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    procedure: ProcedureDto,
) -> Result<Vec<u8>> {
    let client = get_client(collection, snapshot_path, client)?;
    client
        .execute_procedure(StrongholdProcedure::from(procedure))
        .map(Into::into)
        .map_err(Into::into)
}

fn get_stronghold(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
) -> Result<iota_stronghold::Stronghold> {
    let collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.get(&snapshot_path) {
        Ok(stronghold.inner().clone())
    } else {
        Err(Error::StrongholdNotInitialized)
    }
}

fn get_client(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
) -> Result<Client> {
    let collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.get(&snapshot_path) {
        stronghold.get_client(client).map_err(Into::into)
    } else {
        Err(Error::StrongholdNotInitialized)
    }
}

enum PasswordHashFunctionKind {
    #[cfg(feature = "kdf")]
    Argon2(PathBuf),
    Custom(Box<PasswordHashFn>),
}

pub struct Builder {
    password_hash_function: PasswordHashFunctionKind,
}

impl Builder {
    pub fn new<F: Fn(&str) -> Vec<u8> + Send + Sync + 'static>(password_hash_function: F) -> Self {
        Self {
            password_hash_function: PasswordHashFunctionKind::Custom(Box::new(
                password_hash_function,
            )),
        }
    }

    /// Initializes [`Self`] with argon2 as password hash function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tauri::Manager;
    /// tauri::Builder::default()
    ///     .setup(|app| {
    ///         let salt_path = app
    ///             .path()
    ///             .app_local_data_dir()
    ///             .expect("could not resolve app local data path")
    ///             .join("salt.txt");
    ///         app.handle().plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;
    ///         Ok(())
    ///     });
    /// ```
    #[cfg(feature = "kdf")]
    pub fn with_argon2(salt_path: &std::path::Path) -> Self {
        Self {
            password_hash_function: PasswordHashFunctionKind::Argon2(salt_path.to_owned()),
        }
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let password_hash_function = self.password_hash_function;

        let plugin_builder = PluginBuilder::new("stronghold").setup(move |app, _api| {
            app.manage(StrongholdCollection::default());
            app.manage(PasswordHashFunction(match password_hash_function {
                #[cfg(feature = "kdf")]
                PasswordHashFunctionKind::Argon2(path) => {
                    Box::new(move |p| kdf::KeyDerivation::argon2(p, &path))
                }
                PasswordHashFunctionKind::Custom(f) => f,
            }));
            Ok(())
        });

        Builder::invoke_stronghold_handlers_and_build(plugin_builder)
    }

    fn invoke_stronghold_handlers_and_build<R: Runtime>(
        builder: PluginBuilder<R>,
    ) -> TauriPlugin<R> {
        builder
            .invoke_handler(tauri::generate_handler![
                initialize,
                destroy,
                save,
                create_client,
                load_client,
                get_store_record,
                save_store_record,
                remove_store_record,
                save_secret,
                remove_secret,
                execute_procedure,
            ])
            .build()
    }
}
