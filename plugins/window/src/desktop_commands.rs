// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, Serializer};
use tauri::{
    utils::config::WindowConfig, AppHandle, CursorIcon, Icon, Manager, Monitor, PhysicalPosition,
    PhysicalSize, Position, Runtime, Size, Theme, UserAttentionType, Window,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("window not found")]
    WindowNotFound,
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum IconDto {
    #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
    File(std::path::PathBuf),
    #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
    Raw(Vec<u8>),
    Rgba {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    },
}

impl From<IconDto> for Icon {
    fn from(icon: IconDto) -> Self {
        match icon {
            #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
            IconDto::File(path) => Self::File(path),
            #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
            IconDto::Raw(raw) => Self::Raw(raw),
            IconDto::Rgba {
                rgba,
                width,
                height,
            } => Self::Rgba {
                rgba,
                width,
                height,
            },
        }
    }
}

#[tauri::command]
pub fn create<R: Runtime>(app: AppHandle<R>, options: WindowConfig) -> Result<()> {
    tauri::window::WindowBuilder::from_config(&app, options).build()?;
    Ok(())
}

fn get_window<R: Runtime>(window: Window<R>, label: Option<String>) -> Result<Window<R>> {
    match label {
        Some(l) if !l.is_empty() => window.get_window(&l).ok_or(Error::WindowNotFound),
        _ => Ok(window),
    }
}

macro_rules! getter {
    ($cmd: ident, $ret: ty) => {
        #[tauri::command]
        pub fn $cmd<R: Runtime>(window: Window<R>, label: Option<String>) -> Result<$ret> {
            get_window(window, label)?.$cmd().map_err(Into::into)
        }
    };
}

macro_rules! setter {
    ($cmd: ident) => {
        #[tauri::command]
        pub fn $cmd<R: Runtime>(window: Window<R>, label: Option<String>) -> Result<()> {
            get_window(window, label)?.$cmd().map_err(Into::into)
        }
    };

    ($cmd: ident, $input: ty) => {
        #[tauri::command]
        pub fn $cmd<R: Runtime>(
            window: Window<R>,
            label: Option<String>,
            value: $input,
        ) -> Result<()> {
            get_window(window, label)?.$cmd(value).map_err(Into::into)
        }
    };
}

getter!(scale_factor, f64);
getter!(inner_position, PhysicalPosition<i32>);
getter!(outer_position, PhysicalPosition<i32>);
getter!(inner_size, PhysicalSize<u32>);
getter!(outer_size, PhysicalSize<u32>);
getter!(is_fullscreen, bool);
getter!(is_minimized, bool);
getter!(is_maximized, bool);
getter!(is_decorated, bool);
getter!(is_resizable, bool);
getter!(is_visible, bool);
getter!(title, String);
getter!(current_monitor, Option<Monitor>);
getter!(primary_monitor, Option<Monitor>);
getter!(available_monitors, Vec<Monitor>);
getter!(theme, Theme);

setter!(center);
setter!(request_user_attention, Option<UserAttentionType>);
setter!(set_resizable, bool);
setter!(set_title, &str);
setter!(maximize);
setter!(unmaximize);
setter!(minimize);
setter!(unminimize);
setter!(show);
setter!(hide);
setter!(close);
setter!(set_decorations, bool);
setter!(set_shadow, bool);
setter!(set_always_on_top, bool);
setter!(set_content_protected, bool);
setter!(set_size, Size);
setter!(set_min_size, Option<Size>);
setter!(set_max_size, Option<Size>);
setter!(set_position, Position);
setter!(set_fullscreen, bool);
setter!(set_focus);
setter!(set_skip_taskbar, bool);
setter!(set_cursor_grab, bool);
setter!(set_cursor_visible, bool);
setter!(set_cursor_icon, CursorIcon);
setter!(set_cursor_position, Position);
setter!(set_ignore_cursor_events, bool);
setter!(start_dragging);
setter!(print);

#[tauri::command]
pub fn set_icon<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
    value: IconDto,
) -> Result<()> {
    get_window(window, label)?
        .set_icon(value.into())
        .map_err(Into::into)
}

#[tauri::command]
pub fn toggle_maximize<R: Runtime>(window: Window<R>, label: Option<String>) -> Result<()> {
    let window = get_window(window, label)?;
    match window.is_maximized()? {
        true => window.unmaximize()?,
        false => window.maximize()?,
    };
    Ok(())
}

#[tauri::command]
pub fn internal_toggle_maximize<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
) -> Result<()> {
    let window = get_window(window, label)?;
    if window.is_resizable()? {
        match window.is_maximized()? {
            true => window.unmaximize()?,
            false => window.maximize()?,
        };
    }
    Ok(())
}

#[cfg(any(debug_assertions, feature = "devtools"))]
#[tauri::command]
pub fn internal_toggle_devtools<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
) -> Result<()> {
    let window = get_window(window, label)?;
    if window.is_devtools_open() {
        window.close_devtools();
    } else {
        window.open_devtools();
    }
    Ok(())
}
