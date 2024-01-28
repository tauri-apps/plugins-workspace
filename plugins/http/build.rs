// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[path = "src/scope.rs"]
#[allow(dead_code)]
mod scope;

const COMMANDS: &[&str] = &["fetch", "fetch_cancel", "fetch_send", "fetch_read_body"];

/// HTTP scope entry object definition.
#[derive(schemars::JsonSchema)]
struct ScopeEntry {
    /// A URL that can be accessed by the webview when using the HTTP APIs.
    /// The scoped URL is matched against the request URL using a glob pattern.
    ///
    /// Examples:
    ///
    /// - "https://*" or "https://**" : allows all HTTPS urls
    ///
    /// - "https://*.github.com/tauri-apps/tauri": allows any subdomain of "github.com" with the "tauri-apps/api" path
    ///
    /// - "https://myapi.service.com/users/*": allows access to any URLs that begins with "https://myapi.service.com/users/"
    url: String,
}

fn main() {
    // ensure scope entry is up to date
    let dummy = ScopeEntry {
        url: "https://tauri.app/*".into(),
    };
    let _ = scope::Entry {
        url: dummy.url.parse().unwrap(),
    };

    tauri_plugin::Builder::new(COMMANDS)
        .global_scope_schema(schemars::schema_for!(ScopeEntry))
        .build();
}
