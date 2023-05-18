// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use error::Error;
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

#[derive(Serialize, Clone)]
struct ChangePayload<'a> {
    path: &'a Path,
    key: &'a str,
    value: &'a JsonValue,
}

#[derive(Default)]
pub struct StoreCollection<R: Runtime> {
    stores: Mutex<HashMap<PathBuf, Store<R>>>,
    frozen: bool,
}

pub fn with_store<R: Runtime, T, F: FnOnce(&mut Store<R>) -> Result<T, Error>>(
    app: AppHandle<R>,
    collection: State<'_, StoreCollection<R>>,
    path: impl AsRef<Path>,
    f: F,
) -> Result<T, Error> {
    let mut stores = collection.stores.lock().expect("mutex poisoned");

    let path = path.as_ref();
    if !stores.contains_key(path) {
        if collection.frozen {
            return Err(Error::NotFound(path.to_path_buf()));
        }
        let mut store = StoreBuilder::new(app, path.to_path_buf()).build();
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
) -> Result<(), Error> {
    with_store(app, stores, path, |store| store.insert(key, value))
}

#[tauri::command]
async fn get<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
    key: String,
) -> Result<Option<JsonValue>, Error> {
    with_store(app, stores, path, |store| Ok(store.get(key).cloned()))
}

#[tauri::command]
async fn has<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
    key: String,
) -> Result<bool, Error> {
    with_store(app, stores, path, |store| Ok(store.has(key)))
}

#[tauri::command]
async fn delete<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
    key: String,
) -> Result<bool, Error> {
    with_store(app, stores, path, |store| store.delete(key))
}

#[tauri::command]
async fn clear<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<(), Error> {
    with_store(app, stores, path, |store| store.clear())
}

#[tauri::command]
async fn reset<R: Runtime>(
    app: AppHandle<R>,
    collection: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<(), Error> {
    with_store(app, collection, path, |store| store.reset())
}

#[tauri::command]
async fn keys<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<Vec<String>, Error> {
    with_store(app, stores, path, |store| {
        Ok(store.keys().cloned().collect())
    })
}

#[tauri::command]
async fn values<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<Vec<JsonValue>, Error> {
    with_store(app, stores, path, |store| {
        Ok(store.values().cloned().collect())
    })
}

#[tauri::command]
async fn entries<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<Vec<(String, JsonValue)>, Error> {
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
) -> Result<usize, Error> {
    with_store(app, stores, path, |store| Ok(store.len()))
}

#[tauri::command]
async fn load<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<(), Error> {
    with_store(app, stores, path, |store| store.load())
}

#[tauri::command]
async fn save<R: Runtime>(
    app: AppHandle<R>,
    stores: State<'_, StoreCollection<R>>,
    path: PathBuf,
) -> Result<(), Error> {
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
    pub fn store(mut self, store: Store<R>) -> Self {
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
    pub fn build(mut self) -> TauriPlugin<R> {
        plugin::Builder::new("store")
            .invoke_handler(tauri::generate_handler![
                set, get, has, delete, clear, reset, keys, values, length, entries, load, save
            ])
            .setup(move |app_handle| {
                for (path, store) in self.stores.iter_mut() {
                    // ignore loading errors, just use the default
                    if let Err(err) = store.load() {
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
