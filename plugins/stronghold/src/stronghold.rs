// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types to load, modify and persist an [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs)
//! snapshot file.

use std::{convert::TryFrom, ops::Deref, path::Path};

use iota_stronghold::{KeyProvider, SnapshotPath};
use serde::{Serialize, Serializer};
use zeroize::Zeroizing;

/// Alias for a [`std::result::Result`] with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the stronghold plugin.
///
/// Serialized as the error message string when returned to the frontend.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No stronghold was initialized for the given snapshot path.
    #[error("stronghold not initialized")]
    StrongholdNotInitialized,
    /// An error from the underlying Stronghold client, e.g. when loading a snapshot
    /// with the wrong password or when addressing a client that does not exist.
    #[error(transparent)]
    Stronghold(#[from] iota_stronghold::ClientError),
    /// An error from the Stronghold secure memory implementation, e.g. when the
    /// password hash is not a key size Stronghold accepts.
    #[error(transparent)]
    Memory(#[from] iota_stronghold::MemoryError),
    /// A Stronghold procedure (key generation, key derivation, signing, ...) failed.
    #[error(transparent)]
    Procedure(#[from] iota_stronghold::procedures::ProcedureError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

/// A Stronghold instance bound to a snapshot file and to the key it is encrypted with.
///
/// Dereferences to the underlying [`iota_stronghold::Stronghold`], so all of its client
/// and vault operations are available on this type.
pub struct Stronghold {
    inner: iota_stronghold::Stronghold,
    path: SnapshotPath,
    keyprovider: KeyProvider,
}

impl Stronghold {
    /// Creates a Stronghold instance for the snapshot file at `path`, encrypted with
    /// `password` as the key.
    ///
    /// When the file already exists its snapshot is loaded, which requires `password` to
    /// be the key it was encrypted with. Otherwise an empty instance is created and
    /// nothing is written to disk until [`Self::save`] is called.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Memory`] when `password` is not a key size Stronghold accepts
    /// (it must be 32 bytes long) and [`Error::Stronghold`] when an existing snapshot
    /// cannot be loaded with it.
    pub fn new<P: AsRef<Path>>(path: P, password: Vec<u8>) -> Result<Self> {
        let path = SnapshotPath::from_path(path);
        let stronghold = iota_stronghold::Stronghold::default();
        let keyprovider = KeyProvider::try_from(Zeroizing::new(password))?;
        if path.exists() {
            stronghold.load_snapshot(&keyprovider, &path)?;
        }
        Ok(Self {
            inner: stronghold,
            path,
            keyprovider,
        })
    }

    /// Writes the state of all clients to the snapshot file, encrypted with the key
    /// this instance was created with.
    pub fn save(&self) -> Result<()> {
        self.inner
            .commit_with_keyprovider(&self.path, &self.keyprovider)?;
        Ok(())
    }

    /// Returns a reference to the underlying [`iota_stronghold::Stronghold`] instance.
    pub fn inner(&self) -> &iota_stronghold::Stronghold {
        &self.inner
    }
}

impl Deref for Stronghold {
    type Target = iota_stronghold::Stronghold;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
