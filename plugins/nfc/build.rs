// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &["is_available", "write", "scan"];

fn main() {
    let result = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .android_path("android")
        .ios_path("ios")
        .try_build();

    // when building documentation for Android the plugin build result is always Err() and is irrelevant to the crate documentation build
    if !(cfg!(docsrs) && std::env::var("TARGET").unwrap().contains("android")) {
        result.unwrap();
    }

    // TODO: triple check if this can reference the plugin's xml as it expects rn
    // TODO: This has to be configurable if we want to support handling nfc tags when the app is not open.
    tauri_plugin::mobile::update_android_manifest(
        "NFC PLUGIN",
        "activity",
        r#"<intent-filter>
  <action android:name="android.nfc.action.NDEF_DISCOVERED" />
  <category android:name="android.intent.category.DEFAULT" />
</intent-filter>

<intent-filter>
  <action android:name="android.nfc.action.TECH_DISCOVERED" />
  <category android:name="android.intent.category.DEFAULT" />
</intent-filter>

<intent-filter>
  <action android:name="android.nfc.action.TAG_DISCOVERED" />
  <category android:name="android.intent.category.DEFAULT" />
</intent-filter>

<meta-data
  android:name="android.nfc.action.TECH_DISCOVERED"
  android:resource="@xml/nfc_tech_filter" />"#
            .to_string(),
    )
    .expect("failed to rewrite AndroidManifest.xml");
}
