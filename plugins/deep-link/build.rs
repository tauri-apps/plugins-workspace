// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[path = "src/config.rs"]
mod config;
use config::{AssociatedDomain, Config};

const COMMANDS: &[&str] = &["get_current", "register", "unregister", "is_registered"];

// TODO: Consider using activity-alias in case users may have multiple activities in their app.
fn intent_filter(domain: &AssociatedDomain) -> String {
    let host = domain
    .host
    .as_ref()
    .map(|h| format!(r#"<data android:host="{h}" />"#))
    .unwrap_or_default();

    let auto_verify = if domain.is_app_link() {
        r#"android:autoVerify="true" "#.to_string()
    } else {
        String::new()
    };

    format!(
        r#"<intent-filter {auto_verify}>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    {}
    {}
    {}
    {}
    {}
    {}
</intent-filter>"#,
        domain
            .scheme
            .iter()
            .map(|scheme| format!(r#"<data android:scheme="{scheme}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        host,
        domain
            .path
            .iter()
            .map(|path| format!(r#"<data android:path="{path}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        domain
            .path_pattern
            .iter()
            .map(|pattern| format!(r#"<data android:pathPattern="{pattern}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        domain
            .path_prefix
            .iter()
            .map(|prefix| format!(r#"<data android:pathPrefix="{prefix}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        domain
            .path_suffix
            .iter()
            .map(|suffix| format!(r#"<data android:pathSuffix="{suffix}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
    )
}

fn main() {
    let result = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .android_path("android")
        .try_build();

    // when building documentation for Android the plugin build result is always Err() and is irrelevant to the crate documentation build
    if !(cfg!(docsrs) && std::env::var("TARGET").unwrap().contains("android")) {
        result.unwrap();
    }

    if let Some(config) = tauri_plugin::plugin_config::<Config>("deep-link") {
        let errors: Vec<String> = config
            .mobile
            .iter()
            .filter_map(|d| d.validate().err())
            .collect();

        if !errors.is_empty() {
            panic!("Deep link config validation failed:\n{}", errors.join("\n"));
        }

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


        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            // we need to ensure that the entitlements are only 
            // generated for explicit app links and not 
            // other deep links or web links because then they
            // are just going to complain and not be built or signed
            let has_app_links = config
                .mobile
                .iter()
                .any(|d| d.is_app_link());

            if !has_app_links {
                tauri_plugin::mobile::update_entitlements(|entitlements| {
                    entitlements.remove("com.apple.developer.associated-domains");
                })
                .expect("failed to update entitlements");
            } else {
                tauri_plugin::mobile::update_entitlements(|entitlements| {
                    entitlements.insert(
                        "com.apple.developer.associated-domains".into(),
                        config
                        .mobile
                        .iter()
                        .filter(|d| d.is_app_link())
                        .filter_map(|d| d.host.as_ref())
                        .map(|host| format!("applinks:{}", host).into())
                        .collect::<Vec<_>>()
                        .into(),
                    );
                })
                .expect("failed to update entitlements");
            }
        }
    }
}
