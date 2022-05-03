use tauri::{plugin::TauriPlugin, Runtime};

#[cfg(target_os = "windows")]
#[path = "platform_impl/windows.rs"]
mod platform_impl;
#[cfg(target_os = "linux")]
#[path = "platform_impl/linux.rs"]
mod platform_impl;
#[cfg(target_os = "macos")]
#[path = "platform_impl/macos.rs"]
mod platform_impl;

pub(crate) type SingleInstanceCallback =
    dyn FnMut(Vec<String>, String, Box<dyn FnOnce()>) + Send + Sync + 'static;

pub fn init<
    R: Runtime,
    F: FnMut(Vec<String>, String, Box<dyn FnOnce()>) + Send + Sync + 'static,
>(
    f: F,
) -> TauriPlugin<R> {
    platform_impl::init(Box::new(f))
}
