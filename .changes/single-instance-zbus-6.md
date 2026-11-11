---
single-instance: patch
---

On Linux, updated `zbus` to version 6. The `zvariant`, `zvariant_derive` and `zbus_names` crates, which RustSec flags as unmaintained (RUSTSEC-2026-0370, RUSTSEC-2026-0371 and RUSTSEC-2026-0372), are no longer in the plugin's dependency tree. The plugin's D-Bus connection now runs on Tauri's async runtime, and the callback runs in a task there.

The plugin enables zbus's `tokio` feature, which Cargo then enables for every crate in the app that uses zbus 6. With it, `zbus::block_on` runs on a single-threaded Tokio runtime instead of zbus's own executor. So synchronous APIs built on it, such as `notify-rust`'s `Notification::show`, `close` and `wait_for_action`, panic with "Cannot start a runtime from within a runtime" when called from an async task, and a connection built in `zbus::block_on` is only served while a thread is inside a `zbus::block_on` call.

A current-thread runtime passed to `tauri::async_runtime::set` needs a thread of its own to drive it, and `destroy` must not be called from one of its tasks, including the callback.
