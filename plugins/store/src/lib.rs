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
    sync::Mutex,
};
pub use store::{Store, StoreBuilder};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, RunEvent, Runtime, State,
};

mod error;
mod store;

#[cfg(mobile)]
mod mobile;
#[cfg(mobile)]
use crate::plugin::PluginHandle;
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.store";
#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_store);

#[cfg(desktop)]
mod desktop;

#[derive(Serialize, Clone)]
struct ChangePayload<'a> {
    path: &'a Path,
    key: &'a str,
    value: &'a JsonValue,
}

pub struct StoreCollection<R: Runtime> {
    stores: Mutex<HashMap<PathBuf, Store<R>>>,
    frozen: bool,

    #[cfg(mobile)]
    mobile_plugin_handle: PluginHandle<R>,
}

pub fn with_store<R: Runtime, T, F: FnOnce(&mut Store<R>) -> Result<T>>(
    app: AppHandle<R>,
    collection: State<'_, StoreCollection<R>>,
    path: impl AsRef<Path>,
    f: F,
) -> Result<T> {
    let mut stores = collection.stores.lock().expect("mutex poisoned");

    let path = path.as_ref();
    if !stores.contains_key(path) {
        if collection.frozen {
            return Err(Error::NotFound(path.to_path_buf()));
        }

        #[allow(unused_mut)]
        let mut builder = StoreBuilder::new(path);

        #[cfg(mobile)]
        {
            builder = builder.mobile_plugin_handle(collection.mobile_plugin_handle.clone());
        }

        let mut store = builder.build(app);

        // ignore loading errors, just use the default
        if let Err(err) = store.load() {
            warn!(
                "Failed to load store {:?} from disk: {}. Falling back to default values.",
                path, err
            );
        }
        stores.insert(path.to_path_buf(), store);
    }

    f(stores
        .get_mut(path)
        .expect("failed to retrieve store. This is a bug!"))
}

#[tauri::command]
async fn set<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
    key: String,
    value: JsonValue,
) -> Result<()> {
    with_store(app, stores, path, |store| store.insert(key, value))
}

#[tauri::command]
async fn get<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
    key: String,
) -> Result<Option<JsonValue>> {
    with_store(app, stores, path, |store| Ok(store.get(key).cloned()))
}

#[tauri::command]
async fn has<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
    key: String,
) -> Result<bool> {
    with_store(app, stores, path, |store| Ok(store.has(key)))
}

#[tauri::command]
async fn delete<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
    key: String,
) -> Result<bool> {
    with_store(app, stores, path, |store| store.delete(key))
}

#[tauri::command]
async fn clear<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<()> {
    with_store(app, stores, path, |store| store.clear())
}

#[tauri::command]
async fn reset<R: Runtime>(
    app: AppHandle<R>,
    collection: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<()> {
    with_store(app, collection, path, |store| store.reset())
}

#[tauri::command]
async fn keys<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<Vec<String>> {
    with_store(app, stores, path, |store| {
        Ok(store.keys().cloned().collect())
    })
}

#[tauri::command]
async fn values<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<Vec<JsonValue>> {
    with_store(app, stores, path, |store| {
        Ok(store.values().cloned().collect())
    })
}

#[tauri::command]
async fn entries<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<Vec<(String, JsonValue)>> {
    with_store(app, stores, path, |store| {
        Ok(store
            .entries()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect())
    })
}

#[tauri::command]
async fn length<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<usize> {
    with_store(app, stores, path, |store| Ok(store.len()))
}

#[tauri::command]
async fn load<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<()> {
    with_store(app, stores, path, |store| store.load())
}

#[tauri::command]
async fn save<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<()> {
    with_store(app, stores, path, |store| store.save())
}

// #[derive(Default)]
pub struct Builder<R: Runtime> {
    stores: HashMap<PathBuf, Store<R>>,
    frozen: bool,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            stores: Default::default(),
            frozen: false,
        }
    }
}

impl<R: Runtime> Builder<R> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a store with the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_store::{StoreBuilder, Builder};
    ///
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let store = StoreBuilder::new("store.bin").build(app.handle().clone());
    ///     let builder = Builder::default().store(store);
    ///     Ok(())
    ///   });
    /// ```
    pub fn store(mut self, store: Store<R>) -> Self {
        self.stores.insert(store.path.clone(), store);
        self
    }

    /// Registers multiple stores with the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_store::{StoreBuilder, Builder};
    ///
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let store = StoreBuilder::new("store.bin").build(app.handle().clone());
    ///     let builder = Builder::default().stores([store]);
    ///     Ok(())
    ///   });
    /// ```
    pub fn stores<T: IntoIterator<Item = Store<R>>>(mut self, stores: T) -> Self {
        self.stores = stores
            .into_iter()
            .map(|store| (store.path.clone(), store))
            .collect();
        self
    }

    /// Freezes the collection.
    ///
    /// This causes requests for plugins that haven't been registered to fail
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_store::{StoreBuilder, Builder};
    ///
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let store = StoreBuilder::new("store.bin").build(app.handle().clone());
    ///     app.handle().plugin(Builder::default().freeze().build());
    ///     Ok(())
    ///   });
    /// ```
    pub fn freeze(mut self) -> Self {
        self.frozen = true;
        self
    }

    /// Builds the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_store::{StoreBuilder, Builder};
    ///
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let store = StoreBuilder::new("store.bin").build(app.handle().clone());
    ///     app.handle().plugin(Builder::default().build());
    ///     Ok(())
    ///   });
    /// ```
    pub fn build(mut self) -> TauriPlugin<R> {
        plugin::Builder::new("store")
            .invoke_handler(tauri::generate_handler![
                set, get, has, delete, clear, reset, keys, values, length, entries, load, save
            ])
            .setup(move |app_handle, _api| {
                for (path, store) in self.stores.iter_mut() {
                    // ignore loading errors, just use the default
                    if let Err(err) = store.load() {
                        warn!(
              "Failed to load store {:?} from disk: {}. Falling back to default values.",
              path, err
            );
                    }
                }

                #[cfg(target_os = "android")]
                let handle = _api.register_android_plugin(PLUGIN_IDENTIFIER, "StorePlugin")?;
                #[cfg(target_os = "ios")]
                let handle = _api.register_ios_plugin(init_plugin_store)?;

                app_handle.manage(StoreCollection {
                    stores: Mutex::new(self.stores),
                    frozen: self.frozen,

                    #[cfg(mobile)]
                    mobile_plugin_handle: handle,
                });

                Ok(())
            })
            .on_event(|app_handle, event| {
                if let RunEvent::Exit = event {
                    let collection = app_handle.state::<StoreCollection<R>>();

                    for store in collection.stores.lock().expect("mutex poisoned").values() {
                        if let Err(err) = store.save() {
                            eprintln!("failed to save store {:?} with error {:?}", store.path, err);
                        }
                    }
                }
            })
            .build()
    }
}
