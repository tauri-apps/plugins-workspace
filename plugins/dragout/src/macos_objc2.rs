#![cfg(all(target_os = "macos", feature = "objc2_backend"))]
// macOS native drag-out implementation using objc2 / AppKit wrappers.
// This is a work-in-progress replacement for the previous cocoa/objc implementation.
// Initially it only logs calls; functionality will be ported in subsequent commits.
#![allow(clippy::needless_return)]

use objc2::foundation::{NSString};
use objc2_app_kit::NSApp;

/// One-time initialisation – currently just a log line.
pub fn init() {
    println!("[dragout][objc2] macOS drag-out objc2 backend initialised");
}

/// Start a drag session for a single file; real implementation TBD.
/// For now we only log the request so that the rest of the app continues to work
/// while we incrementally migrate the old cocoa code.
#[allow(unused_variables)]
pub fn start_drag(archive_path: &str, file_path: &str) -> Result<(), String> {
    println!("[dragout][objc2] start_drag called (stub): archive='{}' file='{}'", archive_path, file_path);
    // TODO: Port logic from macos.rs using objc2-foundation / objc2-app-kit
    Err("objc2 drag-out backend not yet implemented".into())
}
