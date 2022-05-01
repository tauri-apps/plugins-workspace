use tauri::{plugin::TauriPlugin, Runtime};

#[cfg(target_os = "windows")]
#[path = "platform_impl/windows.rs"]
mod platform_impl;

pub(crate) type SingleInstanceCallback = dyn FnMut(Vec<String>, String) + Send + 'static;

pub fn init<R: Runtime, F: FnMut(Vec<String>, String) + Send + 'static>(f: F) -> TauriPlugin<R> {
    platform_impl::init(Box::new(f))
}
