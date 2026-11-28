# Experimental OHOS native plugins

This fork adds OHOS backends for clipboard-manager, dialog, fs, notification,
opener and barcode-scanner. It uses the core bridge pinned in
[runtime-pins.json](runtime-pins.json). Stable platform dependencies stay unchanged;
`prepare.py` applies the experimental dependency overlay only in a disposable
build checkout.

The native implementations run inside the Ability, outside the WebView.
Rust calls are queued onto ArkTS through N-API, correlated by request ID, and
resolved back to the existing Tauri plugin APIs. Native errors are propagated.
Ability teardown rejects pending calls, closes native file handles and invalidates
old application handles. Synchronous Rust calls must run off the ArkTS main thread;
the relevant IPC commands use Tauri's async dispatch.

## Supported operations

| Plugin | OHOS backend |
| --- | --- |
| Clipboard | System plain-text write/read and clear. Reading requires native clipboard permission; images/HTML retain the existing mobile API limitations. |
| Dialog | Native message/confirmation dialogs with custom button labels; document/media selection and save picker. Confirmation is never implemented in WebView content. |
| Filesystem | Existing scoped Rust filesystem operations in the sandbox; picker URI open/read/write/seek/stat through owned native descriptors. Only picker-granted URIs are accepted, with read-only media grants preserved. |
| Notification | System permission request/query, immediate text notifications, active notification listing and removal. Scheduled notifications, action types, custom sounds and attachments are not implemented. |
| Opener | System browser/deep-link opening and file viewing. Selecting a specific browser and desktop file-manager reveal are not implemented. |
| Barcode | HarmonyOS ScanKit fullscreen back-camera scanning, format selection, camera permission request/query and app settings. The system scanner's own Back/Cancel control closes it; programmatic cancellation, front-camera and windowed modes are not supported by that API. |

Picker URIs are not ordinary POSIX paths. They are session-scoped and opened via
`@ohos.file.fs`; Rust duplicates the descriptor before releasing its ArkTS owner.
Ordinary paths still go through Tauri's existing path and scope handling.
Exclusive creation and custom Unix flags on provider URIs are rejected rather
than silently weakening their semantics. Directory manipulation, rename and
copy-by-path require ordinary filesystem paths; `fileAccessMode: "copy"` can copy
picked inputs into the private cache first.

ScanKit is Huawei-specific. For a pure OpenHarmony application without ScanKit,
omit the Rust barcode plugin and pass `--without-barcode` to `install.py`.
The other five backends use OpenHarmony APIs. This option excludes the ScanKit
source/import and camera declaration from the generated application.

## Application integration

Use the [tauri-harmony image](https://github.com/LeenHawk/tauri-harmony)
with an application-owned Tauri project. `prepare.py` accepts an external source
directory, the application workspace, and its Tauri host directory:

```bash
python3 shared/ohos/prepare.py /tmp/ohos-plugins-core /path/to/app /path/to/app/src-tauri
```

`prepare.py` clones the pinned runtime outside the workspace and points the six
plugin dependencies to this checkout. After initializing the application's OHOS
project, run `python3 shared/ohos/install.py /path/to/app/src-tauri` to add native
module declarations, Ability hooks, ArkTS backends and permission declarations.
Applications retain ownership of bundle metadata, icons and signing.

## Device validation

Build and sign the application with a profile appropriate to the device and its
permissions. Verify clipboard, confirmation dialogs, file read/save, notifications,
browser opening and QR scanning, including permission denial, dialog cancellation,
and closing the Ability during a pending operation. Device validation remains required.

Clipboard reading and camera use follow the platform's permission and signing
policy. This implementation does not bypass those checks or report success after
a permission failure.

## WebView baseline

`runtime-pins.json` provides the default integration baseline. Applications such as
gproxy and TauriTavern own their pins and pass them to `prepare_sources(destination, pins)`.
Application version constraints are retained; Cargo rejects incompatible fork versions.
The pins select the actual Tauri, Wry, Tao and Ability sources. The Ability
HAR is packaged from the same checkout as the Rust dependency; applications do
not download an independently versioned registry HAR. The Ability fork enables
persistent browser storage, native file inputs/dialogs, external HTTP(S) windows,
and optional host back navigation. Wry keeps main-frame initialization scripts
out of child frames and passes ArkWeb frame metadata to IPC.

`install.py --sources-only` copies the HAR, plugin ArkTS and generated native
module types without changing an application's `entry/oh-package.json5`, Ability or permissions.
The application must already declare its native module types and `vendor/ability.har` dependency.
Device acceptance (including storage across process restarts) is still required.

## Native entry-point dependencies

OHOS applications must directly depend on `napi-ohos = "=1.2.0"` and
`napi-derive-ohos = "=1.2.0"`, matching the runtime fork. The current N-API
procedural macros refer to those crate names in generated application code.
Re-exporting the attribute through Tauri alone does not remove that requirement.
