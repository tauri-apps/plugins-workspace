---
single-instance: patch
---

On Linux, updated `zbus` to version 6. The `zvariant`, `zvariant_derive` and `zbus_names` crates, which RustSec flags as unmaintained (RUSTSEC-2026-0370, RUSTSEC-2026-0371 and RUSTSEC-2026-0372), are no longer in the plugin's dependency tree.

zbus 6 has no blocking API, so the plugin drives the async one with `tauri::async_runtime::block_on`. zbus runs the plugin's D-Bus connection on Tauri's runtime when another crate in the app enables zbus's `tokio` feature (ashpd does by default), and on zbus's own runtime otherwise; the callback runs there too. On zbus's own runtime, calling zbus-based blocking APIs from the callback, such as `notify-rust`'s `Notification::show`, panics, and calling `destroy` from the callback hangs, as before.

When zbus's `tokio` feature is enabled, a current-thread runtime passed to `tauri::async_runtime::set` needs a thread of its own to drive it, and `destroy` must not be called from one of its tasks.
