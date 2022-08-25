#![cfg(target_os = "macos")]

use crate::SingleInstanceCallback;
use tauri::{
    plugin::{self, TauriPlugin},
    Runtime,
};
pub fn init<R: Runtime>(f: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance").build()
}
