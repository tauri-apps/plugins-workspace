# Changelog

## \[2.0.0-alpha.2]

- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.
- [`70e535a`](https://github.com/tauri-apps/plugins-workspace/commit/70e535abd5410873862fb035b6b66f0fea1edde2)([#590](https://github.com/tauri-apps/plugins-workspace/pull/590)) On macOS, fixed tapping on custom title bar doesn't maximize the window.

## \[2.0.0-alpha.1]

- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.
- [`2fc420b`](https://github.com/tauri-apps/plugins-workspace/commit/2fc420ba375de924f236f5b32d26667f742fcd6b)([#418](https://github.com/tauri-apps/plugins-workspace/pull/418)) Add `incognito` window configuration option
- [`84133b5`](https://github.com/tauri-apps/plugins-workspace/commit/84133b57b8c443007c728dd8dbe32b08804009f9)([#426](https://github.com/tauri-apps/plugins-workspace/pull/426)) The window plugin is recieving a few changes to improve consistency and add new features:

  - Removed `appWindow` variable from JS module, use `getCurrent` or `Window.getCurrent`.
  - Removed `WindowManager`, `WebviewWindow` and `WebviewHandle` types and merged them into one `Window` type that matches the name of the rust window type.
  - Added `Window.getCurrent` and `Window.getAll` which is a convenient method for `getCurrent` and `getAll` functions.
- [`c8c3191`](https://github.com/tauri-apps/plugins-workspace/commit/c8c3191565aef518037f9f4519886ca98329fe47)([#392](https://github.com/tauri-apps/plugins-workspace/pull/392)) Added the `setEffects` and `clearEffects` API.

### feat

- [`a79d6d9`](https://github.com/tauri-apps/plugins-workspace/commit/a79d6d94bdbf6d1919adff8e65f79240c31d4a14)([#406](https://github.com/tauri-apps/plugins-workspace/pull/406)) Added the `maximizable`, `minimizable` and `closable` fields on `WindowOptions`.
- [`a79d6d9`](https://github.com/tauri-apps/plugins-workspace/commit/a79d6d94bdbf6d1919adff8e65f79240c31d4a14)([#406](https://github.com/tauri-apps/plugins-workspace/pull/406)) Added the `setMaximizable`, `setMinimizable`, `setClosable`, `isMaximizable`, `isMinimizable` and `isClosable` methods.
- [`83abea3`](https://github.com/tauri-apps/plugins-workspace/commit/83abea3cae8408ce262f3815c1a6cc506e73c486)([#407](https://github.com/tauri-apps/plugins-workspace/pull/407)) Add `WebviewWindow.is_focused` and `WebviewWindow.getFocusedWindow` getters.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
