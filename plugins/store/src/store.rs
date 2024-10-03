// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{ChangePayload, StoreCollection};
use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    fs::{create_dir_all, read, File},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, Resource, ResourceId, Runtime};
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::sleep,
};

type SerializeFn =
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
    serialize_fn: SerializeFn,
    deserialize_fn: DeserializeFn,
    auto_save: Option<Duration>,
}

impl<R: Runtime> StoreBuilder<R> {
    /// Creates a new [`StoreBuilder`].
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let builder = tauri_plugin_store::StoreBuilder::new(app, "store.bin");
    ///     Ok(())
    ///   });
    /// ```
    pub fn new<M: Manager<R>, P: AsRef<Path>>(manager: &M, path: P) -> Self {
        Self {
            app: manager.app_handle().clone(),
            // Since Store.path is only exposed to the user in emit calls we may as well simplify it here already.
            path: dunce::simplified(path.as_ref()).to_path_buf(),
            defaults: None,
            serialize_fn: default_serialize,
            deserialize_fn: default_deserialize,
            auto_save: Some(Duration::from_millis(100)),
        }
    }

    /// Inserts a default key-value pair.
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let mut defaults = std::collections::HashMap::new();
    ///     defaults.insert("foo".to_string(), "bar".into());
    ///
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.bin")
    ///       .defaults(defaults)
    ///       .build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn defaults(mut self, defaults: HashMap<String, JsonValue>) -> Self {
        self.defaults = Some(defaults);
        self
    }

    /// Inserts multiple default key-value pairs.
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.bin")
    ///       .default("foo".to_string(), "bar")
    ///       .build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn default(mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        let key = key.into();
        let value = value.into();
        self.defaults
            .get_or_insert(HashMap::new())
            .insert(key, value);
        self
    }

    /// Defines a custom serialization function.
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.json")
    ///       .serialize(|cache| serde_json::to_vec(&cache).map_err(Into::into))
    ///       .build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn serialize(mut self, serialize: SerializeFn) -> Self {
        self.serialize_fn = serialize;
        self
    }

    /// Defines a custom deserialization function
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.json")
    ///       .deserialize(|bytes| serde_json::from_slice(&bytes).map_err(Into::into))
    ///       .build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn deserialize(mut self, deserialize: DeserializeFn) -> Self {
        self.deserialize_fn = deserialize;
        self
    }

    /// Auto save on modified with a debounce duration
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///    .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.json")
    ///         .auto_save(std::time::Duration::from_millis(100))
    ///         .build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn auto_save(mut self, debounce_duration: Duration) -> Self {
        self.auto_save = Some(debounce_duration);
        self
    }

    /// Disable auto save on modified with a debounce duration
    pub fn disable_auto_save(mut self) -> Self {
        self.auto_save = None;
        self
    }

    pub(crate) fn build_inner(mut self) -> crate::Result<(Arc<Store<R>>, ResourceId)> {
        let collection = self.app.state::<StoreCollection>();
        let mut stores = collection.stores.lock().unwrap();

        if stores.contains_key(&self.path) {
            return Err(crate::Error::AlreadyExists(self.path));
        }

        let mut store_inner = StoreInner::new(
            self.app.clone(),
            self.path.clone(),
            self.defaults.take(),
            self.serialize_fn,
            self.deserialize_fn,
        );
        let _ = store_inner.load();

        let store = Store {
            auto_save: self.auto_save,
            auto_save_debounce_sender: Arc::new(Mutex::new(None)),
            store: Arc::new(Mutex::new(store_inner)),
        };

        let store = Arc::new(store);
        let rid = self.app.resources_table().add_arc(store.clone());
        stores.insert(self.path, rid);

        Ok((store, rid))
    }

    /// Builds the [`Store`].
    ///
    /// # Examples
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.json").build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn build(self) -> crate::Result<Arc<Store<R>>> {
        let (store, _) = self.build_inner()?;
        Ok(store)
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
    pub(crate) defaults: Option<HashMap<String, JsonValue>>,
    pub(crate) serialize_fn: SerializeFn,
    pub(crate) deserialize_fn: DeserializeFn,
}

impl<R: Runtime> StoreInner<R> {
    pub fn new(
        app: AppHandle<R>,
        path: PathBuf,
        defaults: Option<HashMap<String, JsonValue>>,
        serialize_fn: SerializeFn,
        deserialize_fn: DeserializeFn,
    ) -> Self {
        Self {
            app,
            path,
            cache: defaults.clone().unwrap_or_default(),
            defaults,
            serialize_fn,
            deserialize_fn,
        }
    }

    pub fn save(&self) -> crate::Result<()> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .expect("failed to resolve app dir");
        let store_path = app_dir.join(&self.path);

        create_dir_all(store_path.parent().expect("invalid store path"))?;

        let bytes = (self.serialize_fn)(&self.cache).map_err(crate::Error::Serialize)?;
        let mut f = File::create(&store_path)?;
        f.write_all(&bytes)?;

        Ok(())
    }

    /// Update the store from the on-disk state
    pub fn load(&mut self) -> crate::Result<()> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .expect("failed to resolve app dir");
        let store_path = app_dir.join(&self.path);

        let bytes = read(store_path)?;

        self.cache
            .extend((self.deserialize_fn)(&bytes).map_err(crate::Error::Deserialize)?);

        Ok(())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) {
        let key = key.into();
        let value = value.into();
        self.cache.insert(key.clone(), value.clone());
        let _ = self.emit_change_event(&key, &value);
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
        }
        flag
    }

    pub fn clear(&mut self) {
        let keys: Vec<String> = self.cache.keys().cloned().collect();
        self.cache.clear();
        for key in &keys {
            let _ = self.emit_change_event(key, &JsonValue::Null);
        }
    }

    pub fn reset(&mut self) {
        if let Some(defaults) = &self.defaults {
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
    auto_save: Option<Duration>,
    auto_save_debounce_sender: Arc<Mutex<Option<UnboundedSender<AutoSaveMessage>>>>,
    store: Arc<Mutex<StoreInner<R>>>,
}

impl<R: Runtime> Resource for Store<R> {
    fn close(self: Arc<Self>) {
        let store = self.store.lock().unwrap();
        let collection = store.app.state::<StoreCollection>();
        let mut stores = collection.stores.lock().unwrap();
        stores.remove(&store.path);
    }
}

impl<R: Runtime> Store<R> {
    pub fn with_store<T>(&self, f: impl FnOnce(&mut StoreInner<R>) -> T) -> T {
        let mut store = self.store.lock().unwrap();
        f(&mut store)
    }

    pub fn set(&self, key: impl Into<String>, value: impl Into<JsonValue>) {
        self.store.lock().unwrap().insert(key.into(), value.into());
        let _ = self.trigger_auto_save();
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<JsonValue> {
        self.store.lock().unwrap().get(key).cloned()
    }

    pub fn has(&self, key: impl AsRef<str>) -> bool {
        self.store.lock().unwrap().has(key)
    }

    pub fn delete(&self, key: impl AsRef<str>) -> bool {
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
        self.store.lock().unwrap().reset();
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
        self.store.lock().unwrap().load()
    }

    pub fn save(&self) -> crate::Result<()> {
        if let Some(sender) = self.auto_save_debounce_sender.lock().unwrap().take() {
            let _ = sender.send(AutoSaveMessage::Cancel);
        }
        self.store.lock().unwrap().save()
    }

    pub fn close_store(self) {
        let store = self.store.lock().unwrap();
        let app = store.app.clone();
        let collection = app.state::<StoreCollection>();
        let stores = collection.stores.lock().unwrap();
        if let Some(rid) = stores.get(&store.path).copied() {
            drop(store);
            drop(stores);
            let _ = app.resources_table().take::<Store<R>>(rid);
        }
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
                        auto_save_debounce_sender.lock().unwrap().take();
                        let _ = store.lock().unwrap().save();
                        return;
                    }
                };
            }
        });
        Ok(())
    }

    fn apply_pending_auto_save(&self) {
        // Cancel and save if auto save is pending
        if let Some(sender) = self.auto_save_debounce_sender.lock().unwrap().take() {
            let _ = sender.send(AutoSaveMessage::Cancel);
            let _ = self.save();
        };
    }
}

impl<R: Runtime> Drop for Store<R> {
    fn drop(&mut self) {
        self.apply_pending_auto_save();
    }
}
