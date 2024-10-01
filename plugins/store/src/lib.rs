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
use serde::Serialize;
pub use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, Weak},
    time::Duration,
};
pub use store::{Store, StoreBuilder, StoreInner};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, ResourceId, RunEvent, Runtime, Webview,
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
    stores: Mutex<HashMap<PathBuf, Weak<Mutex<StoreInner<R>>>>>,
    // frozen: bool,
}

#[tauri::command]
async fn create_store<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    path: PathBuf,
    auto_save: Option<u64>,
) -> Result<ResourceId> {
    let mut builder = app.store_builder(path);
    if let Some(auto_save) = auto_save {
        builder = builder.auto_save(Duration::from_millis(auto_save));
    }
    let store = builder.build();
    Ok(webview.resources_table().add(store))
}

#[tauri::command]
async fn set<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    key: String,
    value: JsonValue,
) -> Result<()> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    store.set(key, value);
    Ok(())
}

#[tauri::command]
async fn get<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    key: String,
) -> Result<Option<JsonValue>> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    Ok(store.get(key))
}

#[tauri::command]
async fn has<R: Runtime>(webview: Webview<R>, rid: ResourceId, key: String) -> Result<bool> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    Ok(store.has(key))
}

#[tauri::command]
async fn delete<R: Runtime>(webview: Webview<R>, rid: ResourceId, key: String) -> Result<bool> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    Ok(store.delete(key))
}

#[tauri::command]
async fn clear<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> Result<()> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    store.clear();
    Ok(())
}

#[tauri::command]
async fn reset<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> Result<()> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    store.reset();
    Ok(())
}

#[tauri::command]
async fn keys<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> Result<Vec<String>> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    Ok(store.keys())
}

#[tauri::command]
async fn values<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> Result<Vec<JsonValue>> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    Ok(store.values())
}

#[tauri::command]
async fn entries<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
) -> Result<Vec<(String, JsonValue)>> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    Ok(store.entries())
}

#[tauri::command]
async fn length<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> Result<usize> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    Ok(store.length())
}

#[tauri::command]
async fn load<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> Result<()> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    store.load()
}

#[tauri::command]
async fn save<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> Result<()> {
    let store = webview.resources_table().get::<Store<R>>(rid)?;
    store.save()
}

pub trait StoreExt<R: Runtime> {
    fn store(&self, path: impl AsRef<Path>) -> Store<R>;
    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R>;
}

impl<R: Runtime, T: Manager<R>> StoreExt<R> for T {
    fn store(&self, path: impl AsRef<Path>) -> Store<R> {
        StoreBuilder::new(self.app_handle(), path).build()
    }

    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R> {
        StoreBuilder::new(self.app_handle(), path)
    }
}

// #[derive(Default)]
pub struct Builder<R: Runtime> {
    stores: HashMap<PathBuf, Store<R>>,
    // frozen: bool,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            stores: Default::default(),
            // frozen: false,
        }
    }
}

impl<R: Runtime> Builder<R> {
    pub fn new() -> Self {
        Self::default()
    }

    // /// Registers a store with the plugin.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use tauri_plugin_store::{StoreBuilder, Builder};
    // ///
    // /// tauri::Builder::default()
    // ///   .setup(|app| {
    // ///     let store = StoreBuilder::new("store.bin").build(app.handle().clone());
    // ///     let builder = Builder::default().store(store);
    // ///     Ok(())
    // ///   });
    // /// ```
    // pub fn store(mut self, store: Store<R>) -> Self {
    //     self.stores.insert(store.path.clone(), store);
    //     self
    // }

    // /// Registers multiple stores with the plugin.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use tauri_plugin_store::{StoreBuilder, Builder};
    // ///
    // /// tauri::Builder::default()
    // ///   .setup(|app| {
    // ///     let store = StoreBuilder::new("store.bin").build(app.handle().clone());
    // ///     let builder = Builder::default().stores([store]);
    // ///     Ok(())
    // ///   });
    // /// ```
    // pub fn stores<T: IntoIterator<Item = Store<R>>>(mut self, stores: T) -> Self {
    //     self.stores = stores
    //         .into_iter()
    //         .map(|store| (store.path.clone(), store))
    //         .collect();
    //     self
    // }

    // /// Freezes the collection.
    // ///
    // /// This causes requests for plugins that haven't been registered to fail
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use tauri_plugin_store::{StoreBuilder, Builder};
    // ///
    // /// tauri::Builder::default()
    // ///   .setup(|app| {
    // ///     let store = StoreBuilder::new("store.bin").build(app.handle().clone());
    // ///     app.handle().plugin(Builder::default().freeze().build());
    // ///     Ok(())
    // ///   });
    // /// ```
    // pub fn freeze(mut self) -> Self {
    //     self.frozen = true;
    //     self
    // }

    /// Builds the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.bin").build();
    ///     Ok(())
    ///   });
    /// ```
    pub fn build(mut self) -> TauriPlugin<R> {
        plugin::Builder::new("store")
            .invoke_handler(tauri::generate_handler![
                create_store,
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
                    // frozen: self.frozen,
                });

                Ok(())
            })
            .on_event(|_app_handle, event| {
                if let RunEvent::Exit = event {
                    // let collection = app_handle.state::<StoreCollection<R>>();

                    // for store in collection.stores.lock().expect("mutex poisoned").values_mut() {
                    //     if let Some(sender) = store.auto_save_debounce_sender.take() {
                    //         let _ = sender.send(AutoSaveMessage::Cancel);
                    //     }
                    //     if let Err(err) = store.save() {
                    //         eprintln!("failed to save store {:?} with error {:?}", store.path, err);
                    //     }
                    // }
                }
            })
            .build()
    }
}
