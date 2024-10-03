// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/store/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/store)
//!
//! Simple, persistent key-value store.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

pub use error::{Error, Result};
use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use store::resolve_store_path;
pub use store::{DeserializeFn, SerializeFn, Store, StoreBuilder, StoreInner};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, ResourceId, RunEvent, Runtime, State,
};

mod error;
mod store;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ChangePayload<'a> {
    path: &'a Path,
    resource_id: Option<u32>,
    key: &'a str,
    value: &'a JsonValue,
}

pub struct StoreCollection {
    stores: Mutex<HashMap<PathBuf, ResourceId>>,
    serialize_fns: HashMap<String, SerializeFn>,
    deserialize_fns: HashMap<String, DeserializeFn>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum AutoSave {
    DebounceDuration(u64),
    Bool(bool),
}

#[tauri::command]
async fn create_store<R: Runtime>(
    app: AppHandle<R>,
    store_collection: State<'_, StoreCollection>,
    path: PathBuf,
    auto_save: Option<AutoSave>,
    serialize_fn_name: Option<String>,
    deserialize_fn_name: Option<String>,
) -> Result<ResourceId> {
    let mut builder = app.store_builder(path.clone());

    if let Some(auto_save) = auto_save {
        match auto_save {
            AutoSave::DebounceDuration(duration) => {
                builder = builder.auto_save(Duration::from_millis(duration));
            }
            AutoSave::Bool(false) => {
                builder = builder.disable_auto_save();
            }
            _ => {}
        }
    }

    if let Some(serialize_fn_name) = serialize_fn_name {
        let serialize_fn = store_collection
            .serialize_fns
            .get(&serialize_fn_name)
            .ok_or_else(|| crate::Error::SerializeFunctionNotFound(serialize_fn_name))?;
        builder = builder.serialize(*serialize_fn);
    }

    if let Some(deserialize_fn_name) = deserialize_fn_name {
        let deserialize_fn = store_collection
            .deserialize_fns
            .get(&deserialize_fn_name)
            .ok_or_else(|| crate::Error::DeserializeFunctionNotFound(deserialize_fn_name))?;
        builder = builder.deserialize(*deserialize_fn);
    }

    let (_, rid) = builder.build_inner()?;
    Ok(rid)
}

#[tauri::command]
async fn get_store<R: Runtime>(
    app: AppHandle<R>,
    store_collection: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<Option<ResourceId>> {
    let stores = store_collection.stores.lock().unwrap();
    Ok(stores.get(&resolve_store_path(app, path)).copied())
}

#[tauri::command]
async fn close_store<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    Ok(app.resources_table().close(rid)?)
}

#[tauri::command]
async fn set<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
    key: String,
    value: JsonValue,
) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.set(key, value);
    Ok(())
}

#[tauri::command]
async fn get<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
    key: String,
) -> Result<Option<JsonValue>> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.get(key))
}

#[tauri::command]
async fn has<R: Runtime>(app: AppHandle<R>, rid: ResourceId, key: String) -> Result<bool> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.has(key))
}

#[tauri::command]
async fn delete<R: Runtime>(app: AppHandle<R>, rid: ResourceId, key: String) -> Result<bool> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.delete(key))
}

#[tauri::command]
async fn clear<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.clear();
    Ok(())
}

#[tauri::command]
async fn reset<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.reset();
    Ok(())
}

#[tauri::command]
async fn keys<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<Vec<String>> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.keys())
}

#[tauri::command]
async fn values<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<Vec<JsonValue>> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.values())
}

#[tauri::command]
async fn entries<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
) -> Result<Vec<(String, JsonValue)>> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.entries())
}

#[tauri::command]
async fn length<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<usize> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.length())
}

#[tauri::command]
async fn load<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.load()
}

#[tauri::command]
async fn save<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.save()
}

pub trait StoreExt<R: Runtime> {
    /// Create a store or get an existing store with default settings at path
    fn store(&self, path: impl AsRef<Path>) -> Arc<Store<R>>;
    /// Create a store with default settings
    fn create_store(&self, path: impl AsRef<Path>) -> Result<Arc<Store<R>>>;
    /// Get a store builder
    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R>;
    /// Get an existing store
    fn get_store(&self, path: impl AsRef<Path>) -> Option<Arc<Store<R>>>;
}

impl<R: Runtime, T: Manager<R>> StoreExt<R> for T {
    fn store(&self, path: impl AsRef<Path>) -> Arc<Store<R>> {
        self.get_store(&path)
            .unwrap_or_else(|| self.create_store(&path).unwrap())
    }

    fn create_store(&self, path: impl AsRef<Path>) -> Result<Arc<Store<R>>> {
        StoreBuilder::new(self.app_handle(), path).build()
    }

    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R> {
        StoreBuilder::new(self.app_handle(), path)
    }

    fn get_store(&self, path: impl AsRef<Path>) -> Option<Arc<Store<R>>> {
        let collection = self.state::<StoreCollection>();
        let stores = collection.stores.lock().unwrap();
        stores
            .get(path.as_ref())
            .and_then(|rid| self.resources_table().get(*rid).ok())
    }
}

pub struct Builder<R: Runtime> {
    phantom_data: PhantomData<R>,
    serialize_fns: HashMap<String, SerializeFn>,
    deserialize_fns: HashMap<String, DeserializeFn>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            phantom_data: Default::default(),
            serialize_fns: Default::default(),
            deserialize_fns: Default::default(),
        }
    }
}

impl<R: Runtime> Builder<R> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a serialize function to access it from the JavaScript side
    pub fn register_serialize_fn(mut self, name: String, serialize_fn: SerializeFn) -> Self {
        self.serialize_fns.insert(name, serialize_fn);
        self
    }

    /// Register a deserialize function to access it from the JavaScript side
    pub fn register_deserialize_fn(mut self, name: String, deserialize_fn: DeserializeFn) -> Self {
        self.deserialize_fns.insert(name, deserialize_fn);
        self
    }

    /// Builds the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.bin").build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn build(self) -> TauriPlugin<R> {
        plugin::Builder::new("store")
            .invoke_handler(tauri::generate_handler![
                create_store,
                get_store,
                close_store,
                set,
                get,
                has,
                delete,
                clear,
                reset,
                keys,
                values,
                length,
                entries,
                load,
                save,
            ])
            .setup(move |app_handle, _api| {
                app_handle.manage(StoreCollection {
                    stores: Mutex::new(HashMap::new()),
                    serialize_fns: self.serialize_fns,
                    deserialize_fns: self.deserialize_fns,
                });
                Ok(())
            })
            .on_event(|app_handle, event| {
                if let RunEvent::Exit = event {
                    let collection = app_handle.state::<StoreCollection>();
                    let stores = collection.stores.lock().unwrap();
                    for (path, rid) in stores.iter() {
                        if let Ok(store) = app_handle.resources_table().get::<Store<R>>(*rid) {
                            if let Err(err) = store.save() {
                                log::error!("failed to save store {path:?} with error {err:?}");
                            }
                        }
                    }
                }
            })
            .build()
    }
}
