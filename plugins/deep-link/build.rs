// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::exit;

#[derive(Clone, serde::Deserialize)]
struct Config {
    android: Vec<AndroidConfig>,
}

#[derive(Clone, serde::Deserialize)]
struct AndroidConfig {
    domain: String,
    #[serde(rename = "pathPrefix")]
    path_prefix: Option<String>,
}

// TODO: Consider using activity-alias in case users may have multiple activities in their app.
// TODO: Do we need multiple pathPrefixes? Do we want to support the other path* configs too?
fn intent_filter(domain: &str, path: Option<&str>) -> String {
    format!(
        r#"<intent-filter android:autoVerify="true">
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="http" />
    <data android:scheme="https" />
    <data android:host="{domain}" />{}
</intent-filter>"#,
        if let Some(path) = path {
            format!("\n    <data android:pathPrefix=\"{path}\" />")
        } else {
            String::new()
        }
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
                .android
                .iter()
                .map(|e| intent_filter(&e.domain, e.path_prefix.as_deref()))
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .expect("failed to rewrite AndroidManifest.xml");

        /* #[cfg(target_os = "macos")]
        {
            tauri_build::mobile::update_entitlements(|entitlements| {
                entitlements.insert(
                    "com.apple.developer.associated-domains".into(),
                    config
                        .domains
                        .into_iter()
                        .map(Into::into)
                        .collect::<Vec<_>>()
                        .into(),
                );
            })
            .expect("failed to update entitlements");
        } */
    }
}
