---
deep-link: minor
deep-link-js: minor
opener: minor
opener-js: minor
single-instance: patch
updater: patch
updater-js: patch
---

Updated `windows-rs` dependencies:

- deep-link: updated `windows-registry` to 0.6 and `windows-result` to 0.4. The public `Error::Windows` variant wraps `windows_result::Error`; applications using this type directly should update their `windows-result` dependency.
- opener: updated `windows` to 0.62. The public `Error::Win32Error` variant wraps `windows::core::Error`; applications using this type directly should update their `windows` dependency.
- single-instance: updated `windows-sys` to 0.61
- updater: updated `windows-sys` to 0.61
