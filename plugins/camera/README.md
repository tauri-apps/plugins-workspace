# Camera Plugin

Prompt the user to take a photo using the camera or pick an image from the gallery. Mobile only.

## Install

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

```toml
[dependencies]
tauri-plugin-camera = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "feat/camera" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
npm install 'https://gitpkg.now.sh/tauri-apps/plugins-workspace/plugins/camera?feat/camera'
# or
yarn add 'https://gitpkg.now.sh/tauri-apps/plugins-workspace/plugins/camera?feat/camera'
```

**NOT AVAILABLE YET, WILL BE READY WHEN WE MERGE THE BRANCH:**
```sh
pnpm add https://github.com/tauri-apps/tauri-plugin-camera
# or
npm add https://github.com/tauri-apps/tauri-plugin-camera
# or
yarn add https://github.com/tauri-apps/tauri-plugin-camera
```

## Usage

Register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_camera::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { getPhoto } from "tauri-plugin-camera-api";
const image = await getPhoto();
```

### Android

Add the following permissions on the `src-tauri/gen/android/$(APPNAME)/app/src/main/AndroidManifest.xml` file:

```xml
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE"/>
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
```

### iOS

Configure the following properties on `src-tauri/gen/apple/$(APPNAME)_iOS/Info.plist`:

```xml
<key>NSCameraUsageDescription</key>
<string>Description for the camera usage here</string>
<key>NSPhotoLibraryAddUsageDescription</key>
<string>Description for the library add usage here</string>
<key>NSPhotoLibraryUsageDescription</key>
<string>Description for the library usage here</string>
```

