// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &[
    "load",
    "get_store",
    "set",
    "get",
    "has",
    "delete",
    "clear",
    "reset",
    "keys",
    "values",
    "entries",
    "length",
    "reload",
    "save",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .build();

    // The tests create a tauri app, and with tauri's `common-controls-v6` feature (enabled by
    // other workspace members, e.g. on `cargo test --workspace`) it imports comctl32 v6 functions
    // such as `TaskDialogIndirect`. Without a manifest selecting that version Windows loads v5 and
    // the test binary fails to start with STATUS_ENTRYPOINT_NOT_FOUND. tauri-build embeds this
    // manifest for apps, test binaries need it from here. Only affects this crate's own tests.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}
