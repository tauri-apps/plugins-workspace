[package]
name = "tauri-plugin-{{name}}"
version = "1.0.0"
edition = { workspace = true }
authors = { workspace = true }
license = { workspace = true }
links = "tauri-plugin-{{name}}"

[package.metadata.docs.rs]
rustc-args = [ "--cfg", "docsrs" ]
rustdoc-args = [ "--cfg", "docsrs" ]

[build-dependencies]
tauri-plugin = { workspace = true, features = [ "build" ] }

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
tauri = { workspace = true }
log = { workspace = true }
thiserror = { workspace = true }
