// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{ChangePayload, Error};
use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    fs::{create_dir_all, read, File},
    io::Write,
    path::PathBuf,
};
use tauri::{AppHandle, Manager, Runtime};

type SerializeFn =
    fn(&HashMap<String, JsonValue>) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
type DeserializeFn =
    fn(&[u8]) -> Result<HashMap<String, JsonValue>, Box<dyn std::error::Error + Send + Sync>>;

fn default_serialize(
    cache: &HashMap<String, JsonValue>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::to_vec(&cache)?)
}

fn default_deserialize(
    bytes: &[u8],
) -> Result<HashMap<String, JsonValue>, Box<dyn std::error::Error + Send + Sync>> {
    serde_json::from_slice(bytes).map_err(Into::into)
}

/// Builds a [`Store`]
pub struct StoreBuilder<R: Runtime> {
    app: AppHandle<R>,
    path: PathBuf,
    defaults: Option<HashMap<String, JsonValue>>,
    cache: HashMap<String, JsonValue>,
    serialize: SerializeFn,
    deserialize: DeserializeFn,
}

impl<R: Runtime> StoreBuilder<R> {
    /// Creates a new [`StoreBuilder`].
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    ///
    /// let builder = StoreBuilder::new("store.bin".parse()?);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(app: AppHandle<R>, path: PathBuf) -> Self {
        Self {
            app,
            path,
            defaults: None,
            cache: Default::default(),
            serialize: default_serialize,
            deserialize: default_deserialize,
        }
    }

    /// Inserts a default key-value pair.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    /// use std::collections::HashMap;
    ///
    /// let mut defaults = HashMap::new();
    ///
    /// defaults.insert("foo".to_string(), "bar".into());
    ///
    /// let builder = StoreBuilder::new("store.bin".parse()?)
    ///   .defaults(defaults);
    ///
    /// # Ok(())
    /// # }
    pub fn defaults(mut self, defaults: HashMap<String, JsonValue>) -> Self {
        self.cache = defaults.clone();
        self.defaults = Some(defaults);
        self
    }

    /// Inserts multiple key-value pairs.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    ///
    /// let builder = StoreBuilder::new("store.bin".parse()?)
    ///   .default("foo".to_string(), "bar".into());
    ///
    /// # Ok(())
    /// # }
    pub fn default(mut self, key: String, value: JsonValue) -> Self {
        self.cache.insert(key.clone(), value.clone());
        self.defaults
            .get_or_insert(HashMap::new())
            .insert(key, value);
        self
    }

    /// Defines a custom serialization function.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    ///
    /// let builder = StoreBuilder::new("store.json".parse()?)
    ///   .serialize(|cache| serde_json::to_vec(&cache).map_err(Into::into));
    ///
    /// # Ok(())
    /// # }
    pub fn serialize(mut self, serialize: SerializeFn) -> Self {
        self.serialize = serialize;
        self
    }

    /// Defines a custom deserialization function
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    ///
    /// let builder = StoreBuilder::new("store.json".parse()?)
    ///   .deserialize(|bytes| serde_json::from_slice(&bytes).map_err(Into::into));
    ///
    /// # Ok(())
    /// # }
    pub fn deserialize(mut self, deserialize: DeserializeFn) -> Self {
        self.deserialize = deserialize;
        self
    }

    /// Builds the [`Store`].
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    ///
    /// let store = StoreBuilder::new("store.bin".parse()?).build();
    ///
    /// # Ok(())
    /// # }
    pub fn build(self) -> Store<R> {
        Store {
            app: self.app,
            path: self.path,
            defaults: self.defaults,
            cache: self.cache,
            serialize: self.serialize,
            deserialize: self.deserialize,
        }
    }
}

#[derive(Clone)]
pub struct Store<R: Runtime> {
    app: AppHandle<R>,
    pub(crate) path: PathBuf,
    defaults: Option<HashMap<String, JsonValue>>,
    cache: HashMap<String, JsonValue>,
    serialize: SerializeFn,
    deserialize: DeserializeFn,
}

impl<R: Runtime> Store<R> {
    /// Update the store from the on-disk state
    pub fn load(&mut self) -> Result<(), Error> {
        let app_dir = self
            .app
            .path_resolver()
            .app_data_dir()
            .expect("failed to resolve app dir");
        let store_path = app_dir.join(&self.path);

        let bytes = read(store_path)?;

        self.cache
            .extend((self.deserialize)(&bytes).map_err(Error::Deserialize)?);

        Ok(())
    }

    /// Saves the store to disk
    pub fn save(&self) -> Result<(), Error> {
        let app_dir = self
            .app
            .path_resolver()
            .app_data_dir()
            .expect("failed to resolve app dir");
        let store_path = app_dir.join(&self.path);

        create_dir_all(store_path.parent().expect("invalid store path"))?;

        let bytes = (self.serialize)(&self.cache).map_err(Error::Serialize)?;
        let mut f = File::create(&store_path)?;
        f.write_all(&bytes)?;

        Ok(())
    }

    pub fn insert(&mut self, key: String, value: JsonValue) -> Result<(), Error> {
        self.cache.insert(key.clone(), value.clone());
        self.app.emit_all(
            "store://change",
            ChangePayload {
                path: &self.path,
                key: &key,
                value: &value,
            },
        )?;

        Ok(())
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&JsonValue> {
        self.cache.get(key.as_ref())
    }

    pub fn has(&self, key: impl AsRef<str>) -> bool {
        self.cache.contains_key(key.as_ref())
    }

    pub fn delete(&mut self, key: impl AsRef<str>) -> Result<bool, Error> {
        let flag = self.cache.remove(key.as_ref()).is_some();
        if flag {
            self.app.emit_all(
                "store://change",
                ChangePayload {
                    path: &self.path,
                    key: key.as_ref(),
                    value: &JsonValue::Null,
                },
            )?;
        }
        Ok(flag)
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        let keys: Vec<String> = self.cache.keys().cloned().collect();
        self.cache.clear();
        for key in keys {
            self.app.emit_all(
                "store://change",
                ChangePayload {
                    path: &self.path,
                    key: &key,
                    value: &JsonValue::Null,
                },
            )?;
        }
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        let has_defaults = self.defaults.is_some();

        if has_defaults {
            if let Some(defaults) = &self.defaults {
                for (key, value) in &self.cache {
                    if defaults.get(key) != Some(value) {
                        let _ = self.app.emit_all(
                            "store://change",
                            ChangePayload {
                                path: &self.path,
                                key,
                                value: defaults.get(key).unwrap_or(&JsonValue::Null),
                            },
                        );
                    }
                }
                self.cache = defaults.clone();
            }
            Ok(())
        } else {
            self.clear()
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.cache.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &JsonValue> {
        self.cache.values()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &JsonValue)> {
        self.cache.iter()
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl<R: Runtime> std::fmt::Debug for Store<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("path", &self.path)
            .field("defaults", &self.defaults)
            .field("cache", &self.cache)
            .finish()
    }
}
