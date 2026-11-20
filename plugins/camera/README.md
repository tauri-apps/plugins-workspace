# Camera

Take a photo with the camera or pick images from the gallery on Android and iOS.

| Platform | Supported |
| -------- | --------- |
| Linux    | x         |
| Windows  | x         |
| macOS    | x         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

_This plugin requires a Rust version of at least **1.90**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-camera = "2.0.0"
# alternatively with Git:
tauri-plugin-camera = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

<!-- Add the branch for installations using git! -->

```sh
pnpm add @tauri-apps/plugin-camera
# or
npm add @tauri-apps/plugin-camera
# or
yarn add @tauri-apps/plugin-camera
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_camera::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Second, add the required permissions in the project:

`src-tauri/capabilities/default.json`

```json
  "permissions": ["camera:default"]
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import {
  getPhoto,
  pickImages,
  ResultType,
  Source
} from '@tauri-apps/plugin-camera'

// ask the user whether to take a picture or pick one from the gallery
const image = await getPhoto({
  source: Source.Prompt,
  resultType: ResultType.DataUrl,
  width: 1024
})
// null if the user cancelled
if (image) {
  document.querySelector('img').src = image.data
}

// let the user pick up to 5 images, which are returned as files
const images = await pickImages({ limit: 5 })
```

The camera app and the gallery of the system are used, so the plugin does not render anything in your app.

Images are returned as JPEG. With `ResultType.Uri` and `pickImages` they are written to the app's cache directory on Android and to its temporary directory on iOS. The system removes those files when it needs the space, so move the ones you want to keep.

### Permissions

Picking images from the gallery needs no permission on either platform. `getPhoto` requests the permissions it needs. To ask ahead of time, use `checkPermissions` and `requestPermissions`:

```javascript
import { checkPermissions, requestPermissions } from '@tauri-apps/plugin-camera'

let { camera } = await checkPermissions()
if (camera === 'prompt' || camera === 'prompt-with-rationale') {
  camera = (await requestPermissions(['camera'])).camera
}
```

### Android

No configuration is required.

The picture is taken by the camera app, which does not need your app to hold the `CAMERA` permission. But if your app declares that permission in its manifest (the [barcode scanner](../barcode-scanner) plugin does, for instance), Android only opens the camera app once the user granted it, so the plugin requests it.

`saveToGallery` needs the `WRITE_EXTERNAL_STORAGE` permission on Android 9 and older, which the plugin declares and requests. If the user denies it, the picture is still returned with `saved` set to `false`.

### iOS

Taking a picture requires the `NSCameraUsageDescription` key and `saveToGallery` requires the `NSPhotoLibraryAddUsageDescription` key in `src-tauri/Info.plist` (or `src-tauri/Info.ios.plist`). The plugin rejects the call if the key it needs is missing.

```xml
<key>NSCameraUsageDescription</key>
<string>Take pictures to attach to your notes</string>
<key>NSPhotoLibraryAddUsageDescription</key>
<string>Save the pictures you take to your library</string>
```

The camera is not available in the Simulator.

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src="https://github.com/tauri-apps/plugins-workspace/raw/v2/.github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
