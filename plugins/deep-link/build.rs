// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[path = "src/config.rs"]
mod config;
use config::{AssociatedDomain, Config};

const COMMANDS: &[&str] = &["get_current", "register", "unregister", "is_registered"];

// TODO: Consider using activity-alias in case users may have multiple activities in their app.
// TODO: Do we want to support the other path* configs too?
fn intent_filter(domain: &AssociatedDomain) -> String {
    format!(
        r#"<intent-filter android:autoVerify="true">
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="http" />
    <data android:scheme="https" />
    <data android:host="{}" />
    {}
</intent-filter>"#,
        domain.host,
        domain
            .path_prefix
            .iter()
            .map(|prefix| format!(r#"<data android:pathPrefix="{prefix}" />"#))
            .collect::<Vec<_>>()
            .join("\n    ")
    )
}

fn main() {
    if let Err(error) = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .android_path("android")
        .try_build()
    {
        println!("{error:#}");
        if !(cfg!(docsrs) && std::env::var("TARGET").unwrap().contains("android")) {
            std::process::exit(1);
        }
    }

    if let Some(config) = tauri_plugin::plugin_config::<Config>("deep-link") {
        tauri_plugin::mobile::update_android_manifest(
            "DEEP LINK PLUGIN",
            "activity",
            config
                .mobile
                .iter()
                .map(intent_filter)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .expect("failed to rewrite AndroidManifest.xml");

        #[cfg(target_os = "macos")]
        {
            tauri_plugin::mobile::update_entitlements(|entitlements| {
                entitlements.insert(
                    "com.apple.developer.associated-domains".into(),
                    config
                        .mobile
                        .into_iter()
                        .map(|d| format!("applinks:{}", d.host).into())
                        .collect::<Vec<_>>()
                        .into(),
                );
            })
            .expect("failed to update entitlements");
        }
    }
}
