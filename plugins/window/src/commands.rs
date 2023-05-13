#![allow(unused_imports, dead_code)]

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

#[cfg(feature = "allow-create")]
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

#[allow(unused_macros)]
macro_rules! getter {
    ($cmd: ident, $ret: ty) => {
        #[tauri::command]
        pub fn $cmd<R: Runtime>(window: Window<R>, label: Option<String>) -> Result<$ret> {
            get_window(window, label)?.$cmd().map_err(Into::into)
        }
    };
}

#[allow(unused_macros)]
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

#[cfg(feature = "allow-scale-factor")]
getter!(scale_factor, f64);
#[cfg(feature = "allow-inner-position")]
getter!(inner_position, PhysicalPosition<i32>);
#[cfg(feature = "allow-outer-position")]
getter!(outer_position, PhysicalPosition<i32>);
#[cfg(feature = "allow-inner-size")]
getter!(inner_size, PhysicalSize<u32>);
#[cfg(feature = "allow-outer-size")]
getter!(outer_size, PhysicalSize<u32>);
#[cfg(feature = "allow-is-fullscreen")]
getter!(is_fullscreen, bool);
#[cfg(feature = "allow-is-minimized")]
getter!(is_minimized, bool);
#[cfg(feature = "allow-is-maximized")]
getter!(is_maximized, bool);
#[cfg(feature = "allow-is-decorated")]
getter!(is_decorated, bool);
#[cfg(feature = "allow-is-resizable")]
getter!(is_resizable, bool);
#[cfg(feature = "allow-is-visible")]
getter!(is_visible, bool);
#[cfg(feature = "allow-title")]
getter!(title, String);
#[cfg(feature = "allow-current-monitor")]
getter!(current_monitor, Option<Monitor>);
#[cfg(feature = "allow-primary-monitor")]
getter!(primary_monitor, Option<Monitor>);
#[cfg(feature = "allow-available-monitors")]
getter!(available_monitors, Vec<Monitor>);
#[cfg(feature = "allow-theme")]
getter!(theme, Theme);

#[cfg(feature = "allow-center")]
setter!(center);
#[cfg(feature = "allow-request-user-attention")]
setter!(request_user_attention, Option<UserAttentionType>);
#[cfg(feature = "allow-set-resizable")]
setter!(set_resizable, bool);
#[cfg(feature = "allow-set-title")]
setter!(set_title, &str);
#[cfg(feature = "allow-maximize")]
setter!(maximize);
#[cfg(feature = "allow-unmaximize")]
setter!(unmaximize);
#[cfg(feature = "allow-minimize")]
setter!(minimize);
#[cfg(feature = "allow-unminimize")]
setter!(unminimize);
#[cfg(feature = "allow-show")]
setter!(show);
#[cfg(feature = "allow-hide")]
setter!(hide);
#[cfg(feature = "allow-close")]
setter!(close);
#[cfg(feature = "allow-set-decorations")]
setter!(set_decorations, bool);
#[cfg(feature = "allow-set-shadow")]
setter!(set_shadow, bool);
#[cfg(feature = "allow-set-always-on-top")]
setter!(set_always_on_top, bool);
#[cfg(feature = "allow-set-content-protected")]
setter!(set_content_protected, bool);
#[cfg(feature = "allow-set-size")]
setter!(set_size, Size);
#[cfg(feature = "allow-set-min-size")]
setter!(set_min_size, Option<Size>);
#[cfg(feature = "allow-set-max-size")]
setter!(set_max_size, Option<Size>);
#[cfg(feature = "allow-set-position")]
setter!(set_position, Position);
#[cfg(feature = "allow-set-fullscreen")]
setter!(set_fullscreen, bool);
#[cfg(feature = "allow-set-focus")]
setter!(set_focus);
#[cfg(feature = "allow-set-skip-taskbar")]
setter!(set_skip_taskbar, bool);
#[cfg(feature = "allow-set-cursor-grab")]
setter!(set_cursor_grab, bool);
#[cfg(feature = "allow-set-cursor-visible")]
setter!(set_cursor_visible, bool);
#[cfg(feature = "allow-set-cursor-icon")]
setter!(set_cursor_icon, CursorIcon);
#[cfg(feature = "allow-set-cursor-position")]
setter!(set_cursor_position, Position);
#[cfg(feature = "allow-set-ignore-cursor-events")]
setter!(set_ignore_cursor_events, bool);
#[cfg(feature = "allow-start-dragging")]
setter!(start_dragging);
#[cfg(feature = "allow-print")]
setter!(print);

#[cfg(feature = "allow-set-icon")]
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

#[cfg(feature = "allow-toggle-maximize")]
#[tauri::command]
pub fn toggle_maximize<R: Runtime>(window: Window<R>, label: Option<String>) -> Result<()> {
    let window = get_window(window, label)?;
    match window.is_maximized()? {
        true => window.unmaximize()?,
        false => window.maximize()?,
    };
    Ok(())
}

#[cfg(feature = "allow-toggle-maximize")]
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

#[cfg(debug_assertions)]
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
