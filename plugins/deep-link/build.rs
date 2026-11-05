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
    <!-- ChromeOS ARC++ uses a different action for deep links -->
    <action android:name="org.chromium.arc.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    {schemes}
    {host}
    {domains}
    {path_patterns}
    {path_prefixes}
    {path_suffixes}
</intent-filter>"#,
        schemes = domain
            .scheme
            .iter()
            .map(|scheme| format!(r#"<data android:scheme="{scheme}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        host = host,
        domains = domain
            .path
            .iter()
            .map(|path| format!(r#"<data android:path="{path}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        path_patterns = domain
            .path_pattern
            .iter()
            .map(|pattern| format!(r#"<data android:pathPattern="{pattern}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        path_prefixes = domain
            .path_prefix
            .iter()
            .map(|prefix| format!(r#"<data android:pathPrefix="{prefix}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
        path_suffixes = domain
            .path_suffix
            .iter()
            .map(|suffix| format!(r#"<data android:pathSuffix="{suffix}" />"#))
            .collect::<Vec<_>>()
            .join("\n    "),
    )
    .trim()
    .to_string()
}

fn main() {
    let result = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .android_path("android")
        .try_build();

    // when building documentation for Android the plugin build result is always Err() (docs.rs mounts the sources read-only) and is irrelevant to the crate documentation build
    // `cfg(docsrs)` only reaches rustdoc, build scripts must check the `DOCS_RS` env var instead
    if !(std::env::var_os("DOCS_RS").is_some()
        && std::env::var("TARGET").unwrap().contains("android"))
    {
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
            println!("cargo:rerun-if-env-changed=TAURI_IOS_PROJECT_PATH");
            println!("cargo:rerun-if-env-changed=TAURI_IOS_APP_NAME");
            if let (Some(project_path), Ok(app_name)) = (
                std::env::var_os("TAURI_IOS_PROJECT_PATH"),
                std::env::var("TAURI_IOS_APP_NAME"),
            ) {
                let app_path =
                    std::path::PathBuf::from(project_path).join(format!("{app_name}_iOS"));
                for path in [
                    app_path.join("Info.plist"),
                    app_path.join(format!("{app_name}_iOS.entitlements")),
                ] {
                    // Track the nearest existing parent until generation creates the file.
                    // Watching a missing file directly would rerun the script on every build.
                    if let Some(path) = path.ancestors().find(|path| path.exists()) {
                        println!("cargo:rerun-if-changed={}", path.display());
                    }
                }
            }

            // we need to ensure that the entitlements are only
            // generated for explicit app links and not
            // other deep links because then they
            // are just going to complain and not be built or signed
            tauri_plugin::mobile::update_entitlements(|entitlements| {
                let current = entitlements
                    .get("com.apple.developer.associated-domains")
                    .and_then(|domains| domains.as_array())
                    .map(|domains| {
                        domains
                            .iter()
                            .filter_map(|domain| domain.as_string())
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let associated_domains = config.ios_associated_domains(&current);

                if associated_domains.is_empty() {
                    entitlements.remove("com.apple.developer.associated-domains");
                } else {
                    entitlements.insert(
                        "com.apple.developer.associated-domains".into(),
                        associated_domains
                            .into_iter()
                            .map(Into::into)
                            .collect::<Vec<_>>()
                            .into(),
                    );
                }
            })
            .expect("failed to update entitlements");

            let url_types = config.ios_url_types();

            tauri_plugin::mobile::update_info_plist(|info_plist| {
                if url_types.is_empty() {
                    info_plist.remove("CFBundleURLTypes");
                } else {
                    info_plist.insert(
                        "CFBundleURLTypes".into(),
                        url_types
                            .into_iter()
                            .map(|url_type| {
                                let mut dict = plist::Dictionary::new();
                                dict.insert(
                                    "CFBundleURLSchemes".into(),
                                    url_type
                                        .schemes
                                        .into_iter()
                                        .map(Into::into)
                                        .collect::<Vec<_>>()
                                        .into(),
                                );
                                dict.insert("CFBundleURLName".into(), url_type.name.into());
                                plist::Value::Dictionary(dict)
                            })
                            .collect::<Vec<_>>()
                            .into(),
                    );
                }
            })
            .expect("failed to update Info.plist");
        }
    }
}
