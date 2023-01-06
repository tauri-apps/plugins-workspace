// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use error::Error;
use log::warn;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
pub use store::{Store, StoreBuilder};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, RunEvent, Runtime, State, Window,
};

mod error;
mod store;

#[derive(Serialize, Clone)]
struct ChangePayload {
    path: PathBuf,
    key: String,
    value: JsonValue,
}

#[derive(Default)]
struct StoreCollection {
    stores: Mutex<HashMap<PathBuf, Store>>,
    frozen: bool,
}

fn with_store<R: Runtime, T, F: FnOnce(&mut Store) -> Result<T, Error>>(
    app: &AppHandle<R>,
    collection: State<'_, StoreCollection>,
    path: PathBuf,
    f: F,
) -> Result<T, Error> {
    let mut stores = collection.stores.lock().expect("mutex poisoned");

    if !stores.contains_key(&path) {
        if collection.frozen {
            return Err(Error::NotFound(path));
        }
        let mut store = StoreBuilder::new(path.clone()).build();
        // ignore loading errors, just use the default
        if let Err(err) = store.load(app) {
            warn!(
                "Failed to load store {:?} from disk: {}. Falling back to default values.",
                path, err
            );
        }
        stores.insert(path.clone(), store);
    }

    f(stores
        .get_mut(&path)
        .expect("failed to retrieve store. This is a bug!"))
}

#[tauri::command]
async fn set<R: Runtime>(
    app: AppHandle<R>,
    window: Window<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
    key: String,
    value: JsonValue,
) -> Result<(), Error> {
    with_store(&app, stores, path.clone(), |store| {
        store.cache.insert(key.clone(), value.clone());
        let _ = window.emit("store://change", ChangePayload { path, key, value });
        Ok(())
    })
}

#[tauri::command]
async fn get<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
    key: String,
) -> Result<Option<JsonValue>, Error> {
    with_store(&app, stores, path, |store| {
        Ok(store.cache.get(&key).cloned())
    })
}

#[tauri::command]
async fn has<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
    key: String,
) -> Result<bool, Error> {
    with_store(&app, stores, path, |store| {
        Ok(store.cache.contains_key(&key))
    })
}

#[tauri::command]
async fn delete<R: Runtime>(
    app: AppHandle<R>,
    window: Window<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
    key: String,
) -> Result<bool, Error> {
    with_store(&app, stores, path.clone(), |store| {
        let flag = store.cache.remove(&key).is_some();
        if flag {
            let _ = window.emit(
                "store://change",
                ChangePayload {
                    path,
                    key,
                    value: JsonValue::Null,
                },
            );
        }
        Ok(flag)
    })
}

#[tauri::command]
async fn clear<R: Runtime>(
    app: AppHandle<R>,
    window: Window<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<(), Error> {
    with_store(&app, stores, path.clone(), |store| {
        let keys = store.cache.keys().cloned().collect::<Vec<String>>();
        store.cache.clear();
        for key in keys {
            let _ = window.emit(
                "store://change",
                ChangePayload {
                    path: path.clone(),
                    key,
                    value: JsonValue::Null,
                },
            );
        }
        Ok(())
    })
}

#[tauri::command]
async fn reset<R: Runtime>(
    app: AppHandle<R>,
    window: Window<R>,
    collection: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<(), Error> {
    let has_defaults = collection
        .stores
        .lock()
        .expect("mutex poisoned")
        .get(&path)
        .map(|store| store.defaults.is_some());

    if Some(true) == has_defaults {
        with_store(&app, collection, path.clone(), |store| {
            if let Some(defaults) = &store.defaults {
                for (key, value) in &store.cache {
                    if defaults.get(key) != Some(value) {
                        let _ = window.emit(
                            "store://change",
                            ChangePayload {
                                path: path.clone(),
                                key: key.clone(),
                                value: defaults.get(key).cloned().unwrap_or(JsonValue::Null),
                            },
                        );
                    }
                }
                store.cache = defaults.clone();
            }
            Ok(())
        })
    } else {
        clear(app, window, collection, path).await
    }
}

#[tauri::command]
async fn keys<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<Vec<String>, Error> {
    with_store(&app, stores, path, |store| {
        Ok(store.cache.keys().cloned().collect())
    })
}

#[tauri::command]
async fn values<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<Vec<JsonValue>, Error> {
    with_store(&app, stores, path, |store| {
        Ok(store.cache.values().cloned().collect())
    })
}

#[tauri::command]
async fn entries<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<Vec<(String, JsonValue)>, Error> {
    with_store(&app, stores, path, |store| {
        Ok(store.cache.clone().into_iter().collect())
    })
}

#[tauri::command]
async fn length<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<usize, Error> {
    with_store(&app, stores, path, |store| Ok(store.cache.len()))
}

#[tauri::command]
async fn load<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<(), Error> {
    with_store(&app, stores, path, |store| store.load(&app))
}

#[tauri::command]
async fn save<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection>,
    path: PathBuf,
) -> Result<(), Error> {
    with_store(&app, stores, path, |store| store.save(&app))
}

#[derive(Default)]
pub struct Builder {
    stores: HashMap<PathBuf, Store>,
    frozen: bool,
}

impl Builder {
    /// Registers a store with the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::{StoreBuilder,PluginBuilder};
    ///
    /// let store = StoreBuilder::new("store.bin".parse()?).build();
    ///
    /// let builder = PluginBuilder::default().store(store);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn store(mut self, store: Store) -> Self {
        self.stores.insert(store.path.clone(), store);
        self
    }

    /// Registers multiple stores with the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::{StoreBuilder,PluginBuilder};
    ///
    /// let store = StoreBuilder::new("store.bin".parse()?).build();
    ///
    /// let builder = PluginBuilder::default().stores([store]);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn stores<T: IntoIterator<Item = Store>>(mut self, stores: T) -> Self {
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::{StoreBuilder,PluginBuilder};
    ///
    /// let store = StoreBuilder::new("store.bin".parse()?).build();
    ///
    /// let builder = PluginBuilder::default().freeze();
    ///
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tauri_plugin_store::{StoreBuilder,PluginBuilder};
    /// use tauri::Wry;
    ///
    /// let store = StoreBuilder::new("store.bin".parse()?).build();
    ///
    /// let plugin = PluginBuilder::default().build::<Wry>();
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn build<R: Runtime>(mut self) -> TauriPlugin<R> {
        plugin::Builder::new("store")
            .invoke_handler(tauri::generate_handler![
                set, get, has, delete, clear, reset, keys, values, length, entries, load, save
            ])
            .setup(move |app_handle| {
                for (path, store) in self.stores.iter_mut() {
                    // ignore loading errors, just use the default
                    if let Err(err) = store.load(app_handle) {
                        warn!(
              "Failed to load store {:?} from disk: {}. Falling back to default values.",
              path, err
            );
                    }
                }

                app_handle.manage(StoreCollection {
                    stores: Mutex::new(self.stores),
                    frozen: self.frozen,
                });

                Ok(())
            })
            .on_event(|app_handle, event| {
                if let RunEvent::Exit = event {
                    let collection = app_handle.state::<StoreCollection>();

                    for store in collection.stores.lock().expect("mutex poisoned").values() {
                        if let Err(err) = store.save(app_handle) {
                            eprintln!("failed to save store {:?} with error {:?}", store.path, err);
                        }
                    }
                }
            })
            .build()
    }
}
