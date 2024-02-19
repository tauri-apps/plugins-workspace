// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/global-shortcut/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/global-shortcut)
//!
//! Register global shortcuts.
//!
//! - Supported platforms: Windows, Linux and macOS.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

pub use global_hotkey::hotkey::{Code, HotKey as Shortcut, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri::{
    ipc::Channel,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Manager, Runtime, State, Window,
};

mod error;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;
type HotKeyId = u32;
type HandlerFn<R> = Box<dyn Fn(&AppHandle<R>, &Shortcut) + Send + Sync + 'static>;

enum ShortcutSource {
    Ipc(Channel),
    Rust,
}

impl Clone for ShortcutSource {
    fn clone(&self) -> Self {
        match self {
            Self::Ipc(channel) => Self::Ipc(channel.clone()),
            Self::Rust => Self::Rust,
        }
    }
}

pub struct ShortcutWrapper(Shortcut);

impl From<Shortcut> for ShortcutWrapper {
    fn from(value: Shortcut) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for ShortcutWrapper {
    type Error = global_hotkey::Error;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Shortcut::from_str(value).map(ShortcutWrapper)
    }
}

struct RegisteredShortcut<R: Runtime> {
    source: ShortcutSource,
    shortcut: Shortcut,
    channel_id: Option<String>,
    handler: Option<Arc<HandlerFn<R>>>,
}

pub struct GlobalShortcut<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    manager: std::result::Result<GlobalHotKeyManager, global_hotkey::Error>,
    shortcuts: Arc<Mutex<HashMap<HotKeyId, RegisteredShortcut<R>>>>,
}

impl<R: Runtime> GlobalShortcut<R> {
    fn register_internal_without_handler(
        &self,
        shortcut: Shortcut,
        channel_id: Option<String>,
        source: ShortcutSource,
    ) -> Result<()> {
        let id = shortcut.id();
        acquire_manager(&self.manager)?.register(shortcut)?;
        self.shortcuts.lock().unwrap().insert(
            id,
            RegisteredShortcut {
                source,
                shortcut,
                channel_id,
                handler: None,
            },
        );
        Ok(())
    }

    fn register_internal<F: Fn(&AppHandle<R>, &Shortcut) + Send + Sync + 'static>(
        &self,
        shortcut: Shortcut,
        channel_id: Option<String>,
        source: ShortcutSource,
        handler: F,
    ) -> Result<()> {
        let id = shortcut.id();
        acquire_manager(&self.manager)?.register(shortcut)?;
        self.shortcuts.lock().unwrap().insert(
            id,
            RegisteredShortcut {
                source,
                shortcut,
                channel_id,
                handler: Some(Arc::new(Box::new(handler))),
            },
        );
        Ok(())
    }

    fn register_all_internal_without_handler<S: IntoIterator<Item = (Shortcut, Option<String>)>>(
        &self,
        shortcuts: S,
        source: ShortcutSource,
    ) -> Result<()> {
        let hotkeys = shortcuts
            .into_iter()
            .collect::<Vec<(Shortcut, Option<String>)>>();

        let manager = acquire_manager(&self.manager)?;
        let mut shortcuts = self.shortcuts.lock().unwrap();
        for (shortcut, channel_id) in hotkeys {
            manager.register(shortcut)?;

            shortcuts.insert(
                shortcut.id(),
                RegisteredShortcut {
                    source: source.clone(),
                    shortcut,
                    channel_id,
                    handler: None,
                },
            );
        }

        Ok(())
    }

    fn register_all_internal<S, F>(
        &self,
        shortcuts: S,
        source: ShortcutSource,
        handler: F,
    ) -> Result<()>
    where
        S: IntoIterator<Item = (Shortcut, Option<String>)>,
        F: Fn(&AppHandle<R>, &Shortcut) + Send + Sync + 'static,
    {
        let handler: Arc<HandlerFn<R>> = Arc::new(Box::new(handler));

        let hotkeys = shortcuts
            .into_iter()
            .collect::<Vec<(Shortcut, Option<String>)>>();

        let manager = acquire_manager(&self.manager)?;
        let mut shortcuts = self.shortcuts.lock().unwrap();
        for (shortcut, channel_id) in hotkeys {
            manager.register(shortcut)?;

            shortcuts.insert(
                shortcut.id(),
                RegisteredShortcut {
                    source: source.clone(),
                    shortcut,
                    channel_id,
                    handler: Some(handler.clone()),
                },
            );
        }

        Ok(())
    }

    pub fn register<S, F>(&self, shortcut: S, handler: F) -> Result<()>
    where
        S: TryInto<ShortcutWrapper>,
        S::Error: std::error::Error,
        F: Fn(&AppHandle<R>, &Shortcut) + Send + Sync + 'static,
    {
        self.register_internal(
            try_into_shortcut(shortcut)?,
            None,
            ShortcutSource::Rust,
            handler,
        )
    }

    pub fn register_without_handler<S>(&self, shortcut: S) -> Result<()>
    where
        S: TryInto<ShortcutWrapper>,
        S::Error: std::error::Error,
    {
        self.register_internal_without_handler(
            try_into_shortcut(shortcut)?,
            None,
            ShortcutSource::Rust,
        )
    }

    pub fn register_all<S, T, F>(&self, shortcuts: S, handler: F) -> Result<()>
    where
        S: IntoIterator<Item = T>,
        T: TryInto<ShortcutWrapper>,
        T::Error: std::error::Error,
        F: Fn(&AppHandle<R>, &Shortcut) + Send + Sync + 'static,
    {
        let mut s = Vec::new();
        for shortcut in shortcuts {
            s.push((try_into_shortcut(shortcut)?, None));
        }
        self.register_all_internal(s, ShortcutSource::Rust, handler)
    }

    pub fn register_all_without_handler<S, T>(&self, shortcuts: S) -> Result<()>
    where
        S: IntoIterator<Item = T>,
        T: TryInto<ShortcutWrapper>,
        T::Error: std::error::Error,
    {
        let mut s = Vec::new();
        for shortcut in shortcuts {
            s.push((try_into_shortcut(shortcut)?, None));
        }
        self.register_all_internal_without_handler(s, ShortcutSource::Rust)
    }

    pub fn unregister<S: TryInto<ShortcutWrapper>>(&self, shortcut: S) -> Result<()>
    where
        S::Error: std::error::Error,
    {
        let shortcut = try_into_shortcut(shortcut)?;
        acquire_manager(&self.manager)?.unregister(shortcut)?;

        self.shortcuts.lock().unwrap().remove(&shortcut.id());

        Ok(())
    }

    pub fn unregister_all<T: TryInto<ShortcutWrapper>, S: IntoIterator<Item = T>>(
        &self,
        shortcuts: S,
    ) -> Result<()>
    where
        T::Error: std::error::Error,
    {
        let mut s = Vec::new();
        for shortcut in shortcuts {
            s.push(try_into_shortcut(shortcut)?);
        }

        acquire_manager(&self.manager)?.unregister_all(&s)?;

        let mut shortcuts = self.shortcuts.lock().unwrap();
        for s in s {
            shortcuts.remove(&s.id());
        }

        Ok(())
    }

    /// Determines whether the given shortcut is registered by this application or not.
    ///
    /// If the shortcut is registered by another application, it will still return `false`.
    pub fn is_registered<S: TryInto<ShortcutWrapper>>(&self, shortcut: S) -> bool
    where
        S::Error: std::error::Error,
    {
        if let Ok(shortcut) = try_into_shortcut(shortcut) {
            self.shortcuts.lock().unwrap().contains_key(&shortcut.id())
        } else {
            false
        }
    }
}

pub trait GlobalShortcutExt<R: Runtime> {
    fn global_shortcut(&self) -> &GlobalShortcut<R>;
}

impl<R: Runtime, T: Manager<R>> GlobalShortcutExt<R> for T {
    fn global_shortcut(&self) -> &GlobalShortcut<R> {
        self.state::<GlobalShortcut<R>>().inner()
    }
}

fn acquire_manager(
    manager: &std::result::Result<GlobalHotKeyManager, global_hotkey::Error>,
) -> Result<&GlobalHotKeyManager> {
    manager
        .as_ref()
        .map_err(|e| Error::GlobalHotkey(e.to_string()))
}

fn parse_shortcut<S: AsRef<str>>(shortcut: S) -> Result<Shortcut> {
    shortcut.as_ref().parse().map_err(Into::into)
}

fn try_into_shortcut<S: TryInto<ShortcutWrapper>>(shortcut: S) -> Result<Shortcut>
where
    S::Error: std::error::Error,
{
    shortcut
        .try_into()
        .map(|s| s.0)
        .map_err(|e| Error::GlobalHotkey(e.to_string()))
}

#[tauri::command]
fn register<R: Runtime>(
    _window: Window<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcut: String,
    handler: Channel,
) -> Result<()> {
    global_shortcut.register_internal_without_handler(
        parse_shortcut(&shortcut)?,
        Some(shortcut),
        ShortcutSource::Ipc(handler),
    )
}

#[tauri::command]
fn register_all<R: Runtime>(
    _window: Window<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcuts: Vec<String>,
    handler: Channel,
) -> Result<()> {
    let mut hotkeys = Vec::new();
    for shortcut in shortcuts {
        hotkeys.push((parse_shortcut(&shortcut)?, Some(shortcut)));
    }
    global_shortcut.register_all_internal_without_handler(hotkeys, ShortcutSource::Ipc(handler))
}

#[tauri::command]
fn unregister<R: Runtime>(
    _app: AppHandle<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcut: String,
) -> Result<()> {
    global_shortcut.unregister(parse_shortcut(shortcut)?)
}

#[tauri::command]
fn unregister_all<R: Runtime>(
    _app: AppHandle<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcuts: Vec<String>,
) -> Result<()> {
    let mut hotkeys = Vec::new();
    for shortcut in shortcuts {
        hotkeys.push(parse_shortcut(&shortcut)?);
    }
    global_shortcut.unregister_all(hotkeys)
}

#[tauri::command]
fn is_registered<R: Runtime>(
    _app: AppHandle<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcut: String,
) -> Result<bool> {
    Ok(global_shortcut.is_registered(parse_shortcut(shortcut)?))
}

pub struct Builder<R: Runtime> {
    handler: Option<HandlerFn<R>>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            handler: Default::default(),
        }
    }
}

impl<R: Runtime> Builder<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_handler<F: Fn(&AppHandle<R>, &Shortcut) + Send + Sync + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.handler.replace(Box::new(handler));
        self
    }

    pub fn build(self) -> TauriPlugin<R> {
        let handler = self.handler;
        PluginBuilder::new("global-shortcut")
            .js_init_script(include_str!("api-iife.js").to_string())
            .invoke_handler(tauri::generate_handler![
                register,
                register_all,
                unregister,
                unregister_all,
                is_registered
            ])
            .setup(move |app, _api| {
                let shortcuts =
                    Arc::new(Mutex::new(HashMap::<HotKeyId, RegisteredShortcut<R>>::new()));
                let shortcuts_ = shortcuts.clone();

                let app_handle = app.clone();
                GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
                    if let Some(shortcut) = shortcuts_.lock().unwrap().get(&e.id) {
                        match &shortcut.source {
                            ShortcutSource::Ipc(channel) => {
                                let _ = channel.send(&shortcut.channel_id);
                            }
                            ShortcutSource::Rust => {
                                if let Some(handler) = &shortcut.handler {
                                    handler(&app_handle, &shortcut.shortcut);
                                }
                                if let Some(handler) = &handler {
                                    handler(&app_handle, &shortcut.shortcut);
                                }
                            }
                        }
                    }
                }));

                app.manage(GlobalShortcut {
                    app: app.clone(),
                    manager: GlobalHotKeyManager::new(),
                    shortcuts,
                });
                Ok(())
            })
            .build()
    }
}
