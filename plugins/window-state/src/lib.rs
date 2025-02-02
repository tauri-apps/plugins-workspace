// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Save window positions and sizes and restore them when the app is reopened.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Monitor, PhysicalPosition, PhysicalSize, RunEvent, Runtime, WebviewWindow, Window,
    WindowEvent,
};

use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, File},
    sync::{Arc, Mutex},
};

mod cmd;

type LabelMapperFn = dyn Fn(&str) -> &str + Send + Sync;
type FilterCallbackFn = dyn Fn(&str) -> bool + Send + Sync;

/// Default filename used to store window state.
///
/// If using a custom filename, you should probably use [`AppHandleExt::filename`] instead.
pub const DEFAULT_FILENAME: &str = ".window-state.json";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct StateFlags: u32 {
        const SIZE        = 1 << 0;
        const POSITION    = 1 << 1;
        const MAXIMIZED   = 1 << 2;
        const VISIBLE     = 1 << 3;
        const DECORATIONS = 1 << 4;
        const FULLSCREEN  = 1 << 5;
    }
}

impl Default for StateFlags {
    fn default() -> Self {
        Self::all()
    }
}

struct PluginState {
    filename: String,
    map_label: Option<Box<LabelMapperFn>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct WindowState {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    // prev_x and prev_y are used to store position
    // before maximization happened, because maximization
    // will set x and y to the top-left corner of the monitor
    prev_x: i32,
    prev_y: i32,
    maximized: bool,
    visible: bool,
    decorated: bool,
    fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            x: Default::default(),
            y: Default::default(),
            prev_x: Default::default(),
            prev_y: Default::default(),
            maximized: Default::default(),
            visible: true,
            decorated: true,
            fullscreen: Default::default(),
        }
    }
}

struct WindowStateCache(Arc<Mutex<HashMap<String, WindowState>>>);
/// Used to prevent deadlocks from resize and position event listeners setting the cached state on restoring states
struct RestoringWindowState(Mutex<()>);
pub trait AppHandleExt {
    /// Saves all open windows state to disk
    fn save_window_state(&self, flags: StateFlags) -> Result<()>;
    /// Get the name of the file used to store window state.
    fn filename(&self) -> String;
}

impl<R: Runtime> AppHandleExt for tauri::AppHandle<R> {
    fn save_window_state(&self, flags: StateFlags) -> Result<()> {
        if let Ok(app_dir) = self.path().app_config_dir() {
            let plugin_state = self.state::<PluginState>();
            let state_path = app_dir.join(&plugin_state.filename);
            let windows = self.webview_windows();
            let cache = self.state::<WindowStateCache>();
            let mut state = cache.0.lock().unwrap();

            for (label, s) in state.iter_mut() {
                let window = match &plugin_state.map_label {
                    Some(map) => windows
                        .iter()
                        .find_map(|(l, window)| (map(l) == label).then_some(window)),
                    None => windows.get(label),
                };

                if let Some(window) = window {
                    window.update_state(s, flags)?;
                }
            }

            create_dir_all(&app_dir)
                .map_err(Error::Io)
                .and_then(|_| File::create(state_path).map_err(Into::into))
                .and_then(|mut f| serde_json::to_writer_pretty(&mut f, &*state).map_err(Into::into))
        } else {
            Ok(())
        }
    }

    fn filename(&self) -> String {
        self.state::<PluginState>().filename.clone()
    }
}

pub trait WindowExt {
    /// Restores this window state from disk
    fn restore_state(&self, flags: StateFlags) -> tauri::Result<()>;
}

impl<R: Runtime> WindowExt for WebviewWindow<R> {
    fn restore_state(&self, flags: StateFlags) -> tauri::Result<()> {
        self.as_ref().window().restore_state(flags)
    }
}
impl<R: Runtime> WindowExt for Window<R> {
    fn restore_state(&self, flags: StateFlags) -> tauri::Result<()> {
        let plugin_state = self.app_handle().state::<PluginState>();
        let label = plugin_state
            .map_label
            .as_ref()
            .map(|map| map(self.label()))
            .unwrap_or_else(|| self.label());

        let restoring_window_state = self.state::<RestoringWindowState>();
        let _restoring_window_lock = restoring_window_state.0.lock().unwrap();
        let cache = self.state::<WindowStateCache>();
        let mut c = cache.0.lock().unwrap();

        let mut should_show = true;

        if let Some(state) = c
            .get(label)
            .filter(|state| state != &&WindowState::default())
        {
            if flags.contains(StateFlags::DECORATIONS) {
                self.set_decorations(state.decorated)?;
            }

            if flags.contains(StateFlags::SIZE) {
                self.set_size(PhysicalSize {
                    width: state.width,
                    height: state.height,
                })?;
            }

            if flags.contains(StateFlags::POSITION) {
                let position = (state.x, state.y).into();
                let size = (state.width, state.height).into();
                // restore position to saved value if saved monitor exists
                // otherwise, let the OS decide where to place the window
                for m in self.available_monitors()? {
                    if m.intersects(position, size) {
                        self.set_position(PhysicalPosition {
                            x: if state.maximized {
                                state.prev_x
                            } else {
                                state.x
                            },
                            y: if state.maximized {
                                state.prev_y
                            } else {
                                state.y
                            },
                        })?;
                    }
                }
            }

            if flags.contains(StateFlags::MAXIMIZED) && state.maximized {
                self.maximize()?;
            }

            if flags.contains(StateFlags::FULLSCREEN) {
                self.set_fullscreen(state.fullscreen)?;
            }

            should_show = state.visible;
        } else {
            let mut metadata = WindowState::default();

            if flags.contains(StateFlags::SIZE) {
                let size = self.inner_size()?;
                metadata.width = size.width;
                metadata.height = size.height;
            }

            if flags.contains(StateFlags::POSITION) {
                let pos = self.outer_position()?;
                metadata.x = pos.x;
                metadata.y = pos.y;
            }

            if flags.contains(StateFlags::MAXIMIZED) {
                metadata.maximized = self.is_maximized()?;
            }

            if flags.contains(StateFlags::VISIBLE) {
                metadata.visible = self.is_visible()?;
            }

            if flags.contains(StateFlags::DECORATIONS) {
                metadata.decorated = self.is_decorated()?;
            }

            if flags.contains(StateFlags::FULLSCREEN) {
                metadata.fullscreen = self.is_fullscreen()?;
            }

            c.insert(label.into(), metadata);
        }

        if flags.contains(StateFlags::VISIBLE) && should_show {
            self.show()?;
            self.set_focus()?;
        }

        Ok(())
    }
}

trait WindowExtInternal {
    fn update_state(&self, state: &mut WindowState, flags: StateFlags) -> tauri::Result<()>;
}

impl<R: Runtime> WindowExtInternal for WebviewWindow<R> {
    fn update_state(&self, state: &mut WindowState, flags: StateFlags) -> tauri::Result<()> {
        self.as_ref().window().update_state(state, flags)
    }
}

impl<R: Runtime> WindowExtInternal for Window<R> {
    fn update_state(&self, state: &mut WindowState, flags: StateFlags) -> tauri::Result<()> {
        let is_maximized = flags
            .intersects(StateFlags::MAXIMIZED | StateFlags::POSITION | StateFlags::SIZE)
            && self.is_maximized()?;
        let is_minimized =
            flags.intersects(StateFlags::POSITION | StateFlags::SIZE) && self.is_minimized()?;

        if flags.contains(StateFlags::MAXIMIZED) {
            state.maximized = is_maximized;
        }

        if flags.contains(StateFlags::FULLSCREEN) {
            state.fullscreen = self.is_fullscreen()?;
        }

        if flags.contains(StateFlags::DECORATIONS) {
            state.decorated = self.is_decorated()?;
        }

        if flags.contains(StateFlags::VISIBLE) {
            state.visible = self.is_visible()?;
        }

        if flags.contains(StateFlags::SIZE) && !is_maximized && !is_minimized {
            let size = self.inner_size()?;
            // It doesn't make sense to save a window with 0 height or width
            if size.width > 0 && size.height > 0 {
                state.width = size.width;
                state.height = size.height;
            }
        }

        if flags.contains(StateFlags::POSITION) && !is_maximized && !is_minimized {
            let position = self.outer_position()?;
            state.x = position.x;
            state.y = position.y;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Builder {
    denylist: HashSet<String>,
    filter_callback: Option<Box<FilterCallbackFn>>,
    skip_initial_state: HashSet<String>,
    state_flags: StateFlags,
    map_label: Option<Box<LabelMapperFn>>,
    filename: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the state flags to control what state gets restored and saved.
    pub fn with_state_flags(mut self, flags: StateFlags) -> Self {
        self.state_flags = flags;
        self
    }

    /// Sets a custom filename to use when saving and restoring window states from disk.
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename.replace(filename.into());
        self
    }

    /// Sets a list of windows that shouldn't be tracked and managed by this plugin
    /// For example, splash screen windows.
    pub fn with_denylist(mut self, denylist: &[&str]) -> Self {
        self.denylist = denylist.iter().map(|l| l.to_string()).collect();
        self
    }

    /// Sets a filter callback to exclude specific windows from being tracked.  
    /// Return `true` to save the state, or `false` to skip and not save it.
    pub fn with_filter<F>(mut self, filter_callback: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.filter_callback = Some(Box::new(filter_callback));
        self
    }

    /// Adds the given window label to a list of windows to skip initial state restore.
    pub fn skip_initial_state(mut self, label: &str) -> Self {
        self.skip_initial_state.insert(label.into());
        self
    }

    /// Transforms the window label when saving the window state.
    ///
    /// This can be used to group different windows to use the same state.
    pub fn map_label<F>(mut self, map_fn: F) -> Self
    where
        F: Fn(&str) -> &str + Sync + Send + 'static,
    {
        self.map_label = Some(Box::new(map_fn));
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let flags = self.state_flags;
        let filename = self.filename.unwrap_or_else(|| DEFAULT_FILENAME.into());
        let map_label = self.map_label;

        PluginBuilder::new("window-state")
            .invoke_handler(tauri::generate_handler![
                cmd::save_window_state,
                cmd::restore_state,
                cmd::filename
            ])
            .setup(|app, _api| {
                let cache: Arc<Mutex<HashMap<String, WindowState>>> =
                    if let Ok(app_dir) = app.path().app_config_dir() {
                        let state_path = app_dir.join(&filename);
                        if state_path.exists() {
                            Arc::new(Mutex::new(
                                std::fs::read(state_path)
                                    .map_err(Error::from)
                                    .and_then(|state| {
                                        serde_json::from_slice(&state).map_err(Into::into)
                                    })
                                    .unwrap_or_default(),
                            ))
                        } else {
                            Default::default()
                        }
                    } else {
                        Default::default()
                    };
                app.manage(WindowStateCache(cache));
                app.manage(RestoringWindowState(Mutex::new(())));
                app.manage(PluginState {
                    filename,
                    map_label,
                });
                Ok(())
            })
            .on_window_ready(move |window| {
                let plugin_state = window.app_handle().state::<PluginState>();
                let label = plugin_state
                    .map_label
                    .as_ref()
                    .map(|map| map(window.label()))
                    .unwrap_or_else(|| window.label());

                // Check deny list names
                if self.denylist.contains(label) {
                    return;
                }

                // Check deny list callback
                if let Some(filter_callback) = &self.filter_callback {
                    // Don't save the state if the callback returns false
                    if !filter_callback(label) {
                        return;
                    }
                }

                if !self.skip_initial_state.contains(label) {
                    let _ = window.restore_state(self.state_flags);
                }

                let cache = window.state::<WindowStateCache>();
                let cache = cache.0.clone();
                let label = label.to_string();
                let window_clone = window.clone();
                let flags = self.state_flags;

                // insert a default state if this window should be tracked and
                // the disk cache doesn't have a state for it
                {
                    cache
                        .lock()
                        .unwrap()
                        .entry(label.clone())
                        .or_insert_with(WindowState::default);
                }

                window.on_window_event(move |e| match e {
                    WindowEvent::CloseRequested { .. } => {
                        let mut c = cache.lock().unwrap();
                        if let Some(state) = c.get_mut(&label) {
                            let _ = window_clone.update_state(state, flags);
                        }
                    }

                    WindowEvent::Moved(position) if flags.contains(StateFlags::POSITION) => {
                        if window_clone
                            .state::<RestoringWindowState>()
                            .0
                            .try_lock()
                            .is_ok()
                            && !window_clone.is_minimized().unwrap_or_default()
                        {
                            let mut c = cache.lock().unwrap();
                            if let Some(state) = c.get_mut(&label) {
                                state.prev_x = state.x;
                                state.prev_y = state.y;

                                state.x = position.x;
                                state.y = position.y;
                            }
                        }
                    }
                    WindowEvent::Resized(size) if flags.contains(StateFlags::SIZE) => {
                        if window_clone
                            .state::<RestoringWindowState>()
                            .0
                            .try_lock()
                            .is_ok()
                        {
                            // TODO: Remove once https://github.com/tauri-apps/tauri/issues/5812 is resolved.
                            let is_maximized = if cfg!(target_os = "macos")
                                && (!window_clone.is_decorated().unwrap_or_default()
                                    || !window_clone.is_resizable().unwrap_or_default())
                            {
                                false
                            } else {
                                window_clone.is_maximized().unwrap_or_default()
                            };

                            if !window_clone.is_minimized().unwrap_or_default() && !is_maximized {
                                let mut c = cache.lock().unwrap();
                                if let Some(state) = c.get_mut(&label) {
                                    state.width = size.width;
                                    state.height = size.height;
                                }
                            }
                        }
                    }
                    _ => {}
                });
            })
            .on_event(move |app, event| {
                if let RunEvent::Exit = event {
                    let _ = app.save_window_state(flags);
                }
            })
            .build()
    }
}

trait MonitorExt {
    fn intersects(&self, position: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> bool;
}

impl MonitorExt for Monitor {
    fn intersects(&self, position: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> bool {
        let PhysicalPosition { x, y } = *self.position();
        let PhysicalSize { width, height } = *self.size();

        let left = x;
        let right = x + width as i32;
        let top = y;
        let bottom = y + height as i32;

        [
            (position.x, position.y),
            (position.x + size.width as i32, position.y),
            (position.x, position.y + size.height as i32),
            (
                position.x + size.width as i32,
                position.y + size.height as i32,
            ),
        ]
        .into_iter()
        .any(|(x, y)| x >= left && x < right && y >= top && y < bottom)
    }
}
