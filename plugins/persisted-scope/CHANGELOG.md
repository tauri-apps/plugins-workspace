# Changelog

## \[2.0.0-alpha.0]

- [`ebb2eb2`](https://github.com/tauri-apps/plugins-workspace/commit/ebb2eb2fe2ebfbb70530d16a983d396aa5829aa1)([#274](https://github.com/tauri-apps/plugins-workspace/pull/274)) Recursively unescape saved patterns before allowing/forbidding them. This effectively prevents `.persisted-scope` files from blowing up, which caused Out-Of-Memory issues, while automatically fixing existing broken files seamlessly.
- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!

## \[0.1.1]

- The MSRV was raised to 1.64!
- The plugin now recursively unescapes saved patterns before allowing/forbidding them. This effectively prevents `.persisted-scope` files from blowing up, which caused Out-Of-Memory issues, while automatically fixing existing broken files seamlessly.
  - [ebb2eb2](https://github.com/tauri-apps/plugins-workspace/commit/ebb2eb2fe2ebfbb70530d16a983d396aa5829aa1) fix(persisted-scope): Prevent out-of-memory issues, fixes [#274](https://github.com/tauri-apps/plugins-workspace/pull/274) ([#328](https://github.com/tauri-apps/plugins-workspace/pull/328)) on 2023-04-26
