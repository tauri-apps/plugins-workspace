// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(mobile)]
use crate::plugin::PluginHandle;
use crate::{ChangePayload, StoreCollection};
use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, Resource, Runtime};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::sleep,
};

pub(crate) type SerializeFn =
    fn(&HashMap<String, JsonValue>) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
pub(crate) type DeserializeFn =
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
    pub fn new<P: AsRef<Path>>(app: AppHandle<R>, path: P) -> Self {
        Self {
            app,
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
    /// Note: only works if this store is managed by the plugin (e.g. made using [`crate::with_store`] or inserted into [`crate::Builder`])
    ///
    /// # Examples
    /// ```
    /// use tauri_plugin_store::{Builder, StoreBuilder};
    ///
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let store = StoreBuilder::new("store.json")
    ///         .auto_save(std::time::Duration::from_millis(100))
    ///         .build(app.handle().clone());
    ///     app.handle().plugin(Builder::default().store(store).build())
    ///   });
    /// ```
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
    pub fn build(self) -> Store<R> {
        let collection = self.app.state::<StoreCollection<R>>();
        let mut stores = collection.stores.lock().unwrap();
        let store = stores
            .get(&self.path)
            .and_then(|store| store.upgrade())
            .unwrap_or_else(|| {
                let store = Arc::new(Mutex::new(StoreInner::new(
                    self.app.clone(),
                    self.path.clone(),
                )));
                stores.insert(
                    self.path.clone(),
                    Arc::<Mutex<StoreInner<R>>>::downgrade(&store),
                );
                store
            });
        drop(stores);
        Store {
            defaults: self.defaults,
            serialize: self.serialize,
            deserialize: self.deserialize,
            auto_save: self.auto_save,
            auto_save_debounce_sender: Arc::new(Mutex::new(None)),
            store,

            #[cfg(mobile)]
            mobile_plugin_handle: self.mobile_plugin_handle,
        }
    }
}

pub(crate) enum AutoSaveMessage {
    Reset,
    Cancel,
}

#[derive(Clone)]
pub struct StoreInner<R: Runtime> {
    pub(crate) app: AppHandle<R>,
    pub(crate) path: PathBuf,
    pub(crate) cache: HashMap<String, JsonValue>,

    #[cfg(mobile)]
    pub(crate) mobile_plugin_handle: Option<PluginHandle<R>>,
}

impl<R: Runtime> StoreInner<R> {
    pub fn new(app: AppHandle<R>, path: PathBuf) -> Self {
        Self {
            app,
            path,
            cache: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: JsonValue) {
        self.cache.insert(key.clone(), value.clone());
        let _ = self.emit_change_event(&key, &value);
        // let _ = self.trigger_auto_save();
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&JsonValue> {
        self.cache.get(key.as_ref())
    }

    pub fn has(&self, key: impl AsRef<str>) -> bool {
        self.cache.contains_key(key.as_ref())
    }

    pub fn delete(&mut self, key: impl AsRef<str>) -> bool {
        let flag = self.cache.remove(key.as_ref()).is_some();
        if flag {
            let _ = self.emit_change_event(key.as_ref(), &JsonValue::Null);
            // let _ = self.trigger_auto_save();
        }
        flag
    }

    pub fn clear(&mut self) {
        let keys: Vec<String> = self.cache.keys().cloned().collect();
        self.cache.clear();
        for key in &keys {
            let _ = self.emit_change_event(key, &JsonValue::Null);
        }
        // if !keys.is_empty() {
        //     let _ = self.trigger_auto_save();
        // }
    }

    pub fn reset(&mut self, defaults: &Option<HashMap<String, JsonValue>>) {
        if let Some(defaults) = &defaults {
            for (key, value) in &self.cache {
                if defaults.get(key) != Some(value) {
                    let _ =
                        self.emit_change_event(key, defaults.get(key).unwrap_or(&JsonValue::Null));
                }
            }
            for (key, value) in defaults {
                if !self.cache.contains_key(key) {
                    let _ = self.emit_change_event(key, value);
                }
            }
            self.cache.clone_from(defaults);
            // let _ = self.trigger_auto_save();
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

    fn emit_change_event(&self, key: &str, value: &JsonValue) -> crate::Result<()> {
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
}

impl<R: Runtime> std::fmt::Debug for StoreInner<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("path", &self.path)
            .field("cache", &self.cache)
            .finish()
    }
}

pub struct Store<R: Runtime> {
    defaults: Option<HashMap<String, JsonValue>>,
    serialize: SerializeFn,
    deserialize: DeserializeFn,
    auto_save: Option<Duration>,
    auto_save_debounce_sender: Arc<Mutex<Option<UnboundedSender<AutoSaveMessage>>>>,
    store: Arc<Mutex<StoreInner<R>>>,
}

impl<R: Runtime> Resource for Store<R> {}

impl<R: Runtime> Store<R> {
    pub fn with_store<T>(
        &self,
        f: impl FnOnce(&mut StoreInner<R>) -> crate::Result<T>,
    ) -> crate::Result<T> {
        let mut store = self.store.lock().unwrap();
        f(&mut store)
    }

    pub fn set(&self, key: String, value: JsonValue) {
        self.store.lock().unwrap().insert(key, value);
        let _ = self.trigger_auto_save();
    }

    pub fn get(&self, key: String) -> Option<JsonValue> {
        self.store.lock().unwrap().get(key).cloned()
    }

    pub fn has(&self, key: String) -> bool {
        self.store.lock().unwrap().has(key)
    }

    pub fn delete(&self, key: String) -> bool {
        let deleted = self.store.lock().unwrap().delete(key);
        if deleted {
            let _ = self.trigger_auto_save();
        }
        deleted
    }

    pub fn clear(&self) {
        self.store.lock().unwrap().clear();
        let _ = self.trigger_auto_save();
    }

    pub fn reset(&self) {
        self.store.lock().unwrap().reset(&self.defaults);
        let _ = self.trigger_auto_save();
    }

    pub fn keys(&self) -> Vec<String> {
        self.store.lock().unwrap().keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<JsonValue> {
        self.store.lock().unwrap().values().cloned().collect()
    }

    pub fn entries(&self) -> Vec<(String, JsonValue)> {
        self.store
            .lock()
            .unwrap()
            .entries()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }

    pub fn length(&self) -> usize {
        self.store.lock().unwrap().len()
    }

    pub fn load(&self) -> crate::Result<()> {
        self.store.lock().unwrap().load(self.deserialize)
    }

    pub fn save(&self) -> crate::Result<()> {
        self.store.lock().unwrap().save(self.serialize)
    }

    fn trigger_auto_save(&self) -> crate::Result<()> {
        let Some(auto_save_delay) = self.auto_save else {
            return Ok(());
        };
        if auto_save_delay.is_zero() {
            return self.save();
        }
        let mut auto_save_debounce_sender = self.auto_save_debounce_sender.lock().unwrap();
        if let Some(ref sender) = *auto_save_debounce_sender {
            let _ = sender.send(AutoSaveMessage::Reset);
            return Ok(());
        }
        let (sender, mut receiver) = unbounded_channel();
        auto_save_debounce_sender.replace(sender);
        drop(auto_save_debounce_sender);
        let store = self.store.clone();
        let serialize_fn = self.serialize;
        let auto_save_debounce_sender = self.auto_save_debounce_sender.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                select! {
                    should_cancel = receiver.recv() => {
                        if matches!(should_cancel, Some(AutoSaveMessage::Cancel) | None) {
                            return;
                        }
                    }
                    _ = sleep(auto_save_delay) => {
                        let _ = store.lock().unwrap().save(serialize_fn);
                        auto_save_debounce_sender.lock().unwrap().take();
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
        let auto_save_debounce_sender = self.auto_save_debounce_sender.lock().unwrap();
        if let Some(ref sender) = *auto_save_debounce_sender {
            let _ = sender.send(AutoSaveMessage::Cancel);
        }
    }
}
