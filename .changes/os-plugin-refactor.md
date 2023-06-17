---
"os": minor
"os-js": minor
---

The os plugin is recieving a few changes to improve consistency and add new features:

- Renamed `Kind` enum to `OsType` and `kind()` function to `os_type()`.
- Added `family()`,`exe_extension()`, and `hostname()` functions and their equivalents for JS.
- Removed `tempdir()` function and its equivalent on JS, use `std::env::temp_dir` instead of `temp_dir` from `tauri::path::PathResolver::temp_dir` and `path.tempDir` on JS.
- Modified `platform()` implementation to return `windows` instead of `win32` and `macos` instead of `darwin` to align with Rust's `std::env::consts::OS`
