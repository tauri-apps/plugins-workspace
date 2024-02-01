---
"updater": "patch"
---

Refactored the updater plugin to accommodate to the new changes in tauri config:

- JSON plugin config:

  - Added `pubkey` option.
  - Added `windows` option which is a Windows-specific options.
  - Renamed `installer_args` to `installerArgs`.
  - Moved `installerArgs` option to the new `windows` object.

- Rust crate changes:
  - Added `pubkey` field for `Config`.
  - Added `windows` field for `Config` option which is a Windows-specific options.
  - Added `WindowsConfig` and `WindowsUpdateInstallMode`.
  - Moved `installerArgs` option to the new `windows` object.
  - Added `Builder::pubkey` and `UpdaterBuilder::pubkey`.
  - Changed `Builder::installer_args` and `UpdaterBuilder::installer_args` to add to existing installer args, instead of replacing them, use `Builder/UpdaterBuilder::clear_installer_args` to clear existing args.
  - Added `Builder::installer_arg`, `Builder::clear_installer_args`, `UpdaterBuilder::installer_arg` and `UpdaterBuilder::clear_installer_args`.
