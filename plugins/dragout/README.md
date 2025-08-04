# tauri-plugin-dragout

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-dragout.svg)](https://crates.io/crates/tauri-plugin-dragout)

Native **drag-out** (file promise) support for Tauri applications on **macOS**.

Finder and most Mac apps support dragging a file _out_ of an application before it
physically exists on disk. macOS achieves this with `NSFilePromiseProvider` – the
app promises to create the file later, while the user keeps dragging.

This plugin exposes that capability to Rust-side Tauri code and the JavaScript
frontend.

---

## Features

* Start a drag session for any file that is **inside an archive** or otherwise
  not yet present on disk.
* The file is created lazily in the chosen destination when the user drops it.
* Transparent integration with the system clipboard & drag-and-drop stack.

> **Platform**: macOS 11+. Other platforms are currently _not supported_ – the
> crate purposely fails to compile there.

### Optional BlitzArch backend

When your project is built with the `blitzarch_backend` feature, the plugin uses
[`blitzarch`](https://crates.io/crates/blitzarch) to extract files from
archives on-the-fly. Disable the feature to remove that dependency and provide
your own extraction logic.

```toml
[dependencies]
tauri-plugin-dragout = { version = "0.1", default-features = false }
```

---

## Usage

Rust (main Tauri process):

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dragout::init())
        .invoke_handler(tauri::generate_handler![native_drag_out])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Frontend (React / plain JS):

```ts
await window.__TAURI__.invoke("native_drag_out", {
  archivePath: "/Users/you/Downloads/big.zip",
  filePath: "docs/report.pdf"
});
```

---

## Contributing

PRs and bug reports are welcome! The native part is Objective-C and can be
tricky – tests and sample projects live under `examples/` to help you iterate.

---

## License

Dual-licensed under either of

* MIT License — see `LICENSE-MIT` or <https://opensource.org/licenses/MIT>
* Apache License, Version 2.0 — see `LICENSE-APACHE` or <https://www.apache.org/licenses/LICENSE-2.0>

at your option.
