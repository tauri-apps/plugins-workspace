// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalSize, RunEvent, Runtime, Window,
    WindowEvent,
};

use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, File},
    io::Write,
    sync::{Arc, Mutex},
};

pub const STATE_FILENAME: &str = ".window-state";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    TauriApi(#[from] tauri::api::Error),
    #[error(transparent)]
    Bincode(#[from] Box<bincode::ErrorKind>),
}

/// Defines how the window visibility should be restored.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShowMode {
    /// The window will always be shown, regardless of what the last stored state was.
    Always,
    /// The window will be automatically shown if the last stored state for visibility was `true`.
    LastSaved,
    /// The window will not be automatically shown by this plugin.
    Never,
}

impl Default for ShowMode {
    fn default() -> Self {
        Self::LastSaved
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, Deserialize, Serialize)]
struct WindowMetadata {
    width: f64,
    height: f64,
    x: i32,
    y: i32,
    maximized: bool,
    visible: bool,
    decorated: bool,
    fullscreen: bool,
}

struct WindowStateCache(Arc<Mutex<HashMap<String, WindowMetadata>>>);
pub trait AppHandleExt {
    fn save_window_state(&self) -> Result<()>;
}

impl<R: Runtime> AppHandleExt for tauri::AppHandle<R> {
    fn save_window_state(&self) -> Result<()> {
        if let Some(app_dir) = self.path_resolver().app_config_dir() {
            let state_path = app_dir.join(STATE_FILENAME);
            let cache = self.state::<WindowStateCache>();
            let state = cache.0.lock().unwrap();
            create_dir_all(&app_dir)
                .map_err(Error::Io)
                .and_then(|_| File::create(state_path).map_err(Into::into))
                .and_then(|mut f| {
                    f.write_all(&bincode::serialize(&*state).map_err(Error::Bincode)?)
                        .map_err(Into::into)
                })
        } else {
            Ok(())
        }
    }
}

pub trait WindowExt {
    fn restore_state(&self, show_mode: ShowMode) -> tauri::Result<()>;
}

impl<R: Runtime> WindowExt for Window<R> {
    fn restore_state(&self, show_mode: ShowMode) -> tauri::Result<()> {
        let cache = self.state::<WindowStateCache>();
        let mut c = cache.0.lock().unwrap();
        let mut should_show = true;
        if let Some(state) = c.get(self.label()) {
            self.set_decorations(state.decorated)?;

            self.set_size(LogicalSize {
                width: state.width,
                height: state.height,
            })?;

            // restore position to saved value if saved monitor exists
            // otherwise, let the OS decide where to place the window
            for m in self.available_monitors()? {
                if m.contains((state.x, state.y).into()) {
                    self.set_position(PhysicalPosition {
                        x: state.x,
                        y: state.y,
                    })?;
                }
            }

            if state.maximized {
                self.maximize()?;
            }
            self.set_fullscreen(state.fullscreen)?;

            should_show = state.visible;
        } else {
            let scale_factor = self
                .current_monitor()?
                .map(|m| m.scale_factor())
                .unwrap_or(1.);
            let LogicalSize { width, height } = self.inner_size()?.to_logical(scale_factor);
            let PhysicalPosition { x, y } = self.outer_position()?;
            let maximized = self.is_maximized().unwrap_or(false);
            let visible = self.is_visible().unwrap_or(true);
            let decorated = self.is_decorated().unwrap_or(true);
            let fullscreen = self.is_fullscreen().unwrap_or(false);
            c.insert(
                self.label().into(),
                WindowMetadata {
                    width,
                    height,
                    x,
                    y,
                    maximized,
                    visible,
                    decorated,
                    fullscreen,
                },
            );
        }

        if show_mode == ShowMode::Always || (show_mode == ShowMode::LastSaved && should_show) {
            self.show()?;
            self.set_focus()?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Builder {
    show_mode: ShowMode,
    denylist: HashSet<String>,
    skip_initial_state: HashSet<String>,
}

impl Builder {
    /// Sets how the window visibility should be restored.
    ///
    /// The default is [`ShowMode::LastSaved`]
    pub fn with_show_mode(mut self, show_mode: ShowMode) -> Self {
        self.show_mode = show_mode;
        self
    }

    /// Sets a list of windows that shouldn't be tracked and managed by this plugin
    /// for example splash screen windows.
    pub fn with_denylist(mut self, denylist: &[&str]) -> Self {
        self.denylist = denylist.iter().map(|l| l.to_string()).collect();
        self
    }

    /// Adds the given window label to a list of windows to skip initial state restore.
    pub fn skip_initial_state(mut self, label: &str) -> Self {
        self.skip_initial_state.insert(label.into());
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        PluginBuilder::new("window-state")
            .setup(|app| {
                let cache: Arc<Mutex<HashMap<String, WindowMetadata>>> = if let Some(app_dir) =
                    app.path_resolver().app_config_dir()
                {
                    let state_path = app_dir.join(STATE_FILENAME);
                    if state_path.exists() {
                        Arc::new(Mutex::new(
                            tauri::api::file::read_binary(state_path)
                                .map_err(Error::TauriApi)
                                .and_then(|state| bincode::deserialize(&state).map_err(Into::into))
                                .unwrap_or_default(),
                        ))
                    } else {
                        Default::default()
                    }
                } else {
                    Default::default()
                };
                app.manage(WindowStateCache(cache));
                Ok(())
            })
            .on_webview_ready(move |window| {
                if self.denylist.contains(window.label()) {
                    return;
                }

                if !self.skip_initial_state.contains(window.label()) {
                    let _ = window.restore_state(self.show_mode);
                }

                let cache = window.state::<WindowStateCache>();
                let cache = cache.0.clone();
                let label = window.label().to_string();
                let window_clone = window.clone();
                window.on_window_event(move |e| match e {
                    WindowEvent::Moved(position) => {
                        let mut c = cache.lock().unwrap();
                        if let Some(state) = c.get_mut(&label) {
                            let is_maximized = window_clone.is_maximized().unwrap_or(false);
                            state.maximized = is_maximized;

                            if let Some(monitor) = window_clone.current_monitor().unwrap() {
                                let monitor_position = monitor.position();
                                // save only window positions that are inside the current monitor
                                if position.x > monitor_position.x
                                    && position.y > monitor_position.y
                                    && !is_maximized
                                {
                                    state.x = position.x;
                                    state.y = position.y;
                                };
                            };
                        }
                    }
                    WindowEvent::Resized(size) => {
                        let scale_factor = window_clone
                            .current_monitor()
                            .ok()
                            .map(|m| m.map(|m| m.scale_factor()).unwrap_or(1.))
                            .unwrap_or(1.);
                        let size = size.to_logical(scale_factor);
                        let mut c = cache.lock().unwrap();
                        if let Some(state) = c.get_mut(&label) {
                            let is_maximized = window_clone.is_maximized().unwrap_or(false);
                            let is_fullscreen = window_clone.is_fullscreen().unwrap_or(false);
                            state.decorated = window_clone.is_decorated().unwrap_or(true);
                            state.maximized = is_maximized;
                            state.fullscreen = is_fullscreen;

                            // It doesn't make sense to save a window with 0 height or width
                            if size.width > 0. && size.height > 0. && !is_maximized {
                                state.width = size.width;
                                state.height = size.height;
                            }
                        }
                    }
                    WindowEvent::CloseRequested { .. } => {
                        let mut c = cache.lock().unwrap();
                        if let Some(state) = c.get_mut(&label) {
                            state.visible = window_clone.is_visible().unwrap_or(true);
                        }
                    }
                    _ => {}
                });
            })
            .on_event(|app, event| {
                if let RunEvent::Exit = event {
                    let _ = app.save_window_state();
                }
            })
            .build()
    }
}

trait MonitorExt {
    fn contains(&self, position: PhysicalPosition<i32>) -> bool;
}

impl MonitorExt for Monitor {
    fn contains(&self, position: PhysicalPosition<i32>) -> bool {
        let PhysicalPosition { x, y } = *self.position();
        let PhysicalSize { width, height } = *self.size();

        x < position.x as _
            && position.x < (x + width as i32)
            && y < position.y as _
            && position.y < (y + height as i32)
    }
}
