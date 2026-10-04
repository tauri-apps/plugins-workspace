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

## Build the sample

The repository's [OHOS workflow](../../.github/workflows/ohos.yml) uses the public,
digest-pinned [tauri-harmony image](https://github.com/LeenHawk/tauri-harmony).
It runs four focused URI authorization/handle-lifetime checks, then builds the
Rust and ArkTS code and packages an unsigned ARM64 HAP.

Inside that image, from a disposable checkout:

```bash
python3 shared/ohos/prepare.py /tmp/ohos-plugins-core examples/ohos/src-tauri examples/ohos/src-tauri
export TARGET_TRIPLE=aarch64-unknown-linux-ohos
source /opt/tauri-harmony/env.sh
export PATH="$HARMONY_TOOLS_DIR/command-line-tools/tool/node/bin:$PATH"
(cd examples/ohos/src-tauri && cargo tauri ohos init --ci --skip-targets-install)
python3 shared/ohos/configure-demo.py
python3 shared/ohos/install.py examples/ohos/src-tauri
(cd examples/ohos/src-tauri && cargo tauri ohos build --ci --target aarch64 -- --lib)
```

`prepare.py` clones the pinned core outside the workspace, reuses the image's
compatible Wry/Tao/Ability sources, and points the six plugin dependencies to
this checkout. `install.py` adds the native module declarations, Ability hooks,
ArkTS backends and permission declarations to a freshly initialized HAP project.
Applications keep ownership of bundle metadata, icons and signing; the sample's
`configure-demo.py` is not an application release configuration.

## Device validation

Compilation and the Node boundary checks do not prove device behavior. The latter
exercise the real Files backend with a filesystem double, not the OHOS service.
The sample has manual buttons for clipboard, trusted confirmation, file read/save,
notifications, browser opening and QR scanning. Sign its HAP with a profile
appropriate to the device and declared permissions, then verify those flows,
including permission denial, dialog cancellation, and closing the Ability during
a pending operation. No device/signing environment was available for this change.

Clipboard reading and camera use follow the platform's permission and signing
policy. This implementation does not bypass those checks or report success after
a permission failure.

## WebView baseline

`runtime-pins.json` is the standalone sample baseline only. Applications such as
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
