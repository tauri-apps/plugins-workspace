---
"tauri-plugin-single-instance": patch
---

fix dangling pointers caused by passing `encode_wide().as_ptr()` directly to FFI on Windows.