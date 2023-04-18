use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

pub use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri::{
    api::ipc::CallbackFn,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Manager, Runtime, State, Window,
};

mod error;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;
type HotKeyId = u32;

enum HotKeySource<R: Runtime> {
    Ipc {
        window: Window<R>,
        handler: CallbackFn,
    },
    Rust,
}

impl<R: Runtime> Clone for HotKeySource<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Ipc { window, handler } => Self::Ipc {
                window: window.clone(),
                handler: *handler,
            },
            Self::Rust => Self::Rust,
        }
    }
}

pub struct Shortcut(HotKey);

impl From<HotKey> for Shortcut {
    fn from(value: HotKey) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for Shortcut {
    type Error = global_hotkey::Error;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        HotKey::from_str(value).map(Shortcut)
    }
}

struct RegisteredShortcut<R: Runtime> {
    source: HotKeySource<R>,
    shortcut: (HotKey, Option<String>),
}

pub struct GlobalShortcut<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    manager: std::result::Result<GlobalHotKeyManager, global_hotkey::Error>,
    shortcuts: Arc<Mutex<HashMap<HotKeyId, RegisteredShortcut<R>>>>,
}

impl<R: Runtime> GlobalShortcut<R> {
    fn register_internal(
        &self,
        shortcut: (HotKey, Option<String>),
        source: HotKeySource<R>,
    ) -> Result<()> {
        let id = shortcut.0.id();
        acquire_manager(&self.manager)?.register(shortcut.0.clone())?;
        self.shortcuts
            .lock()
            .unwrap()
            .insert(id, RegisteredShortcut { source, shortcut });
        Ok(())
    }

    fn register_all_internal<S: IntoIterator<Item = (HotKey, Option<String>)>>(
        &self,
        shortcuts: S,
        source: HotKeySource<R>,
    ) -> Result<()> {
        let hotkeys = shortcuts
            .into_iter()
            .collect::<Vec<(HotKey, Option<String>)>>();

        let manager = acquire_manager(&self.manager)?;
        let mut shortcuts = self.shortcuts.lock().unwrap();
        for hotkey in hotkeys {
            manager.register(hotkey.0.clone())?;

            shortcuts.insert(
                hotkey.0.id(),
                RegisteredShortcut {
                    source: source.clone(),
                    shortcut: hotkey,
                },
            );
        }

        Ok(())
    }

    pub fn register<S: TryInto<Shortcut>>(&self, shortcut: S) -> Result<()>
    where
        S::Error: std::error::Error,
    {
        self.register_internal(
            (
                shortcut
                    .try_into()
                    .map(|s| s.0)
                    .map_err(|e| Error::GlobalHotkey(e.to_string()))?,
                None,
            ),
            HotKeySource::Rust,
        )
    }

    pub fn register_all<S: IntoIterator<Item = HotKey>>(&self, shortcuts: S) -> Result<()> {
        self.register_all_internal(shortcuts.into_iter().map(|s| (s, None)), HotKeySource::Rust)
    }

    pub fn unregister(&self, shortcut: HotKey) -> Result<()> {
        acquire_manager(&self.manager)?
            .unregister(shortcut)
            .map_err(Into::into)
    }

    pub fn unregister_all<S: IntoIterator<Item = HotKey>>(&self, shortcuts: S) -> Result<()> {
        acquire_manager(&self.manager)?
            .unregister_all(&shortcuts.into_iter().collect::<Vec<HotKey>>())
            .map_err(Into::into)
    }

    pub fn is_registered(&self, shortcut: HotKey) -> bool {
        self.shortcuts.lock().unwrap().contains_key(&shortcut.id())
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

fn parse_shortcut<S: AsRef<str>>(shortcut: S) -> Result<HotKey> {
    shortcut.as_ref().parse().map_err(Into::into)
}

#[tauri::command]
fn register<R: Runtime>(
    window: Window<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcut: String,
    handler: CallbackFn,
) -> Result<()> {
    global_shortcut.register_internal(
        (parse_shortcut(&shortcut)?, Some(shortcut)),
        HotKeySource::Ipc { window, handler },
    )
}

#[tauri::command]
fn register_all<R: Runtime>(
    window: Window<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcuts: Vec<String>,
    handler: CallbackFn,
) -> Result<()> {
    let mut hotkeys = Vec::new();
    for shortcut in shortcuts {
        hotkeys.push((parse_shortcut(&shortcut)?, Some(shortcut)));
    }
    global_shortcut.register_all_internal(hotkeys, HotKeySource::Ipc { window, handler })
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

#[derive(Default)]
pub struct Builder {
    handler: Option<Box<dyn Fn(&HotKey) + Send + Sync + 'static>>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_handler<F: Fn(&HotKey) + Send + Sync + 'static>(handler: F) -> Self {
        Self {
            handler: Some(Box::new(handler)),
        }
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let handler = self.handler;
        PluginBuilder::new("globalShortcut")
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

                GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
                    if let Some(shortcut) = shortcuts_.lock().unwrap().get(&e.id) {
                        match &shortcut.source {
                            HotKeySource::Ipc { window, handler } => {
                                let callback_string = tauri::api::ipc::format_callback(
                                    *handler,
                                    &shortcut.shortcut.1,
                                )
                                .expect("unable to serialize shortcut string to json");
                                let _ = window.eval(callback_string.as_str());
                            }
                            HotKeySource::Rust => {
                                if let Some(handler) = &handler {
                                    handler(&shortcut.shortcut.0);
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
