// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::exit;

fn main() {
    if let Err(error) = tauri_build::mobile::PluginBuilder::new()
        .android_path("android")
        .ios_path("ios")
        .run()
    {
        println!("{error:#}");
        exit(1);
    }

    // TODO: triple check if this can reference the plugin's xml as it expects rn
    // TODO: This has to be configurable if we want to support handling nfc tags when the app is not open.
    tauri_build::mobile::update_android_manifest(
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
