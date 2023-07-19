// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::process::exit;

#[derive(Deserialize)]
struct AssociatedDomain {
    host: String,
    #[serde(default, alias = "path-prefix", rename = "pathPrefix")]
    path_prefix: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    domains: Vec<AssociatedDomain>,
}

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
    if let Err(error) = tauri_build::mobile::PluginBuilder::new()
        .android_path("android")
        .run()
    {
        println!("{error:#}");
        exit(1);
    }

    if let Some(config) = tauri_build::config::plugin_config::<Config>("deep-link") {
        tauri_build::mobile::update_android_manifest(
            "DEEP LINK PLUGIN",
            "activity",
            config
                .domains
                .iter()
                .map(intent_filter)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .expect("failed to rewrite AndroidManifest.xml");

        #[cfg(target_os = "macos")]
        {
            tauri_build::mobile::update_entitlements(|entitlements| {
                entitlements.insert(
                    "com.apple.developer.associated-domains".into(),
                    config
                        .domains
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
