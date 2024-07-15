// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(mobile)]
use crate::plugin::PluginHandle;
use crate::{ChangePayload, Error, StoreCollection};
use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::sleep,
};

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
    path: PathBuf,
    defaults: Option<HashMap<String, JsonValue>>,
    cache: HashMap<String, JsonValue>,
    serialize: SerializeFn,
    deserialize: DeserializeFn,
    auto_save: Option<Duration>,

    #[cfg(mobile)]
    mobile_plugin_handle: Option<PluginHandle<R>>,
    #[cfg(not(mobile))]
    _marker: std::marker::PhantomData<R>,
}

impl<R: Runtime> StoreBuilder<R> {
    /// Creates a new [`StoreBuilder`].
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    ///
    /// let builder = StoreBuilder::<tauri::Wry>::new("store.bin");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            // Since Store.path is only exposed to the user in emit calls we may as well simplify it here already.
            path: dunce::simplified(path.as_ref()).to_path_buf(),
            defaults: None,
            cache: Default::default(),
            serialize: default_serialize,
            deserialize: default_deserialize,
            auto_save: None,
            #[cfg(mobile)]
            mobile_plugin_handle: None,
            #[cfg(not(mobile))]
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(mobile)]
    pub fn mobile_plugin_handle(mut self, handle: PluginHandle<R>) -> Self {
        self.mobile_plugin_handle = Some(handle);
        self
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
    /// let builder = StoreBuilder::<tauri::Wry>::new("store.bin")
    ///   .defaults(defaults);
    ///
    /// # Ok(())
    /// # }
    pub fn defaults(mut self, defaults: HashMap<String, JsonValue>) -> Self {
        self.cache.clone_from(&defaults);
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
    /// let builder = StoreBuilder::<tauri::Wry>::new("store.bin")
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
    /// let builder = StoreBuilder::<tauri::Wry>::new("store.json")
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
    /// let builder = StoreBuilder::<tauri::Wry>::new("store.json")
    ///   .deserialize(|bytes| serde_json::from_slice(&bytes).map_err(Into::into));
    ///
    /// # Ok(())
    /// # }
    pub fn deserialize(mut self, deserialize: DeserializeFn) -> Self {
        self.deserialize = deserialize;
        self
    }

    /// Auto save on modified with a debounce duration
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::StoreBuilder;
    ///
    /// let builder = StoreBuilder::<tauri::Wry>::new("store.json")
    ///   .auto_save(std::time::Duration::from_millis(100));
    ///
    /// # Ok(())
    /// # }
    pub fn auto_save(mut self, debounce_duration: Duration) -> Self {
        self.auto_save = Some(debounce_duration);
        self
    }

    /// Builds the [`Store`].
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new("store.json").build(app.handle().clone());
    ///     Ok(())
    ///   });
    /// ```
    pub fn build(self, app: AppHandle<R>) -> Store<R> {
        Store {
            app,
            path: self.path,
            defaults: self.defaults,
            cache: self.cache,
            serialize: self.serialize,
            deserialize: self.deserialize,
            auto_save: self.auto_save,
            auto_save_debounce_sender: None,

            #[cfg(mobile)]
            mobile_plugin_handle: self.mobile_plugin_handle,
        }
    }
}

#[derive(Clone)]
pub struct Store<R: Runtime> {
    pub(crate) app: AppHandle<R>,
    pub(crate) path: PathBuf,
    defaults: Option<HashMap<String, JsonValue>>,
    pub(crate) cache: HashMap<String, JsonValue>,
    pub(crate) serialize: SerializeFn,
    pub(crate) deserialize: DeserializeFn,
    pub(crate) auto_save: Option<Duration>,
    pub(crate) auto_save_debounce_sender: Option<UnboundedSender<bool>>,

    #[cfg(mobile)]
    pub(crate) mobile_plugin_handle: Option<PluginHandle<R>>,
}

impl<R: Runtime> Store<R> {
    pub fn insert(&mut self, key: String, value: JsonValue) -> Result<(), Error> {
        self.cache.insert(key.clone(), value.clone());
        self.on_change(&key, &value)?;
        let _ = self.trigger_auto_save();
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
            self.on_change(key.as_ref(), &JsonValue::Null)?;
            let _ = self.trigger_auto_save();
        }
        Ok(flag)
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        let keys: Vec<String> = self.cache.keys().cloned().collect();
        self.cache.clear();
        for key in &keys {
            self.on_change(key, &JsonValue::Null)?;
        }
        if !keys.is_empty() {
            let _ = self.trigger_auto_save();
        }
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        if let Some(defaults) = &self.defaults {
            for (key, value) in &self.cache {
                if defaults.get(key) != Some(value) {
                    let _ = self.on_change(key, defaults.get(key).unwrap_or(&JsonValue::Null));
                }
            }
            self.cache.clone_from(defaults);
            let _ = self.trigger_auto_save();
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

    fn on_change(&self, key: &str, value: &JsonValue) -> Result<(), Error> {
        self.app.emit(
            "store://change",
            ChangePayload {
                path: &self.path,
                key,
                value,
            },
        )?;
        Ok(())
    }

    fn trigger_auto_save(&mut self) -> Result<(), Error> {
        let Some(auto_save_delay) = self.auto_save else {
            return Ok(());
        };
        if auto_save_delay.is_zero() {
            return self.save();
        }
        if let Some(sender) = &self.auto_save_debounce_sender {
            let _ = sender.send(false);
            return Ok(());
        }
        let (sender, mut receiver) = unbounded_channel();
        self.auto_save_debounce_sender.replace(sender);
        let app = self.app.clone();
        let path = self.path.clone();
        tokio::spawn(async move {
            loop {
                select! {
                    should_cancel = receiver.recv() => {
                        if should_cancel == Some(true) {
                            return;
                        }
                    }
                    _ = sleep(auto_save_delay) => {
                        let collection = app.state::<StoreCollection<R>>();
                        if let Some(store) = collection
                            .stores
                            .lock()
                            .expect("mutex poisoned")
                            .values_mut()
                            .find(|store| store.path == path)
                        {
                            let _ = store.save();
                            store.auto_save_debounce_sender = None;
                        }
                        return;
                    }
                };
            }
        });
        Ok(())
    }
}

impl<R: Runtime> Drop for Store<R> {
    fn drop(&mut self) {
        if let Some(sender) = &self.auto_save_debounce_sender {
            let _ = sender.send(false);
        }
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
