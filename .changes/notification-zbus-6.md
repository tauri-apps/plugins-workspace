---
"notification": patch
"notification-js": patch
---

On Linux and the BSDs, updated `zbus` to version 6 and `notify-rust` to 4.19, which is on `zbus` 6. The `zvariant`, `zvariant_derive` and `zbus_names` crates, which RustSec flags as unmaintained (RUSTSEC-2026-0370, RUSTSEC-2026-0371 and RUSTSEC-2026-0372), are no longer in the plugin's dependency tree.

zbus 6 has no blocking API, so the plugin drives the async one with `tauri::async_runtime::block_on`. zbus runs the plugin's D-Bus connection on Tauri's runtime when another crate in the app enables zbus's `tokio` feature (ashpd does by default), and on zbus's own runtime otherwise.
