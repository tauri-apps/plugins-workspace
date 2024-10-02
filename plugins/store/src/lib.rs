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
use log::warn;
use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
    time::Duration,
};
pub use store::{Store, StoreBuilder, StoreInner};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, ResourceId, RunEvent, Runtime,
};

mod error;
mod store;

#[derive(Serialize, Clone)]
struct ChangePayload<'a> {
    path: &'a Path,
    key: &'a str,
    value: &'a JsonValue,
}

pub struct StoreCollection<R: Runtime> {
    /// This weak pointer is always pointing to a real reference since we will remove it on drop
    stores: Mutex<HashMap<PathBuf, (Weak<Store<R>>, Option<ResourceId>)>>,
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
    path: PathBuf,
    auto_save: Option<AutoSave>,
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
    let store = builder.build()?;
    let rid = app.resources_table().add_arc(store);
    let collection = app.state::<StoreCollection<R>>();
    let mut stores = collection.stores.lock().unwrap();
    if let Some((_, resource_id)) = stores.get_mut(&path) {
        resource_id.replace(rid);
    }
    Ok(rid)
}

#[tauri::command]
async fn get_store<R: Runtime>(app: AppHandle<R>, path: PathBuf) -> Option<ResourceId> {
    let collection = app.state::<StoreCollection<R>>();
    let mut stores = collection.stores.lock().unwrap();
    if let Some((store, resource_id)) = stores.get_mut(&path) {
        let rid = if let Some(resource_id) = resource_id {
            *resource_id
        } else {
            let rid = app.resources_table().add_arc(store.upgrade().unwrap());
            resource_id.replace(rid);
            rid
        };
        Some(rid)
    } else {
        None
    }
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
    fn create_store(&self, path: impl AsRef<Path>) -> Result<Arc<Store<R>>>;
    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R>;
    fn share_store(&self, store: Arc<Store<R>>);
    fn get_store(&self, path: impl AsRef<Path>) -> Option<Arc<Store<R>>>;
}

impl<R: Runtime, T: Manager<R>> StoreExt<R> for T {
    fn create_store(&self, path: impl AsRef<Path>) -> Result<Arc<Store<R>>> {
        StoreBuilder::new(self.app_handle(), path).build()
    }

    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R> {
        StoreBuilder::new(self.app_handle(), path)
    }

    fn share_store(&self, store: Arc<Store<R>>) {
        let collection = self.state::<StoreCollection<R>>();
        let mut stores = collection.stores.lock().unwrap();
        if let Some(path) = store.with_store(|inner_store| {
            if stores.contains_key(&inner_store.path) {
                None
            } else {
                Some(inner_store.path.clone())
            }
        }) {
            let weak_store = Arc::downgrade(&store);
            let rid = self.resources_table().add_arc(store);
            stores.insert(path, (weak_store, Some(rid)));
        }
    }

    fn get_store(&self, path: impl AsRef<Path>) -> Option<Arc<Store<R>>> {
        let collection = self.state::<StoreCollection<R>>();
        let stores = collection.stores.lock().unwrap();
        stores
            .get(path.as_ref())
            .and_then(|(store, _)| store.upgrade())
    }
}

// #[derive(Default)]
pub struct Builder<R: Runtime> {
    stores: HashMap<PathBuf, Store<R>>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            stores: Default::default(),
        }
    }
}

impl<R: Runtime> Builder<R> {
    pub fn new() -> Self {
        Self::default()
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
    pub fn build(mut self) -> TauriPlugin<R> {
        plugin::Builder::new("store")
            .invoke_handler(tauri::generate_handler![
                create_store,
                get_store,
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
                save
            ])
            .setup(move |app_handle, _api| {
                for (path, store) in self.stores.iter_mut() {
                    // ignore loading errors, just use the default
                    if let Err(err) = store.load() {
                        warn!(
                            "Failed to load store {path:?} from disk: {err}. Falling back to default values."
                        );
                    }
                }

                app_handle.manage(StoreCollection::<R> {
                    stores: Mutex::new(HashMap::new()),
                });

                Ok(())
            })
            .on_event(|app_handle, event| {
                if let RunEvent::Exit = event {
                    let collection = app_handle.state::<StoreCollection<R>>();
                    let stores = collection.stores.lock().unwrap();
                    for (path, (store, _)) in stores.iter() {
                        let store = store.upgrade().unwrap();
                        if let Err(err) = store.save() {
                            eprintln!("failed to save store {path:?} with error {err:?}");
                        }
                    }
                }
            })
            .build()
    }
}
