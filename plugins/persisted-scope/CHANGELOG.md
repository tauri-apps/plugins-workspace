# Changelog

## \[0.1.1]

- Recursively unescape saved patterns before allowing/forbidding them. This effectively prevents `.persisted-scope` files from blowing up, which caused Out-Of-Memory issues, while automatically fixing existing broken files seamlessly.
  - [ebb2eb2](https://github.com/tauri-apps/plugins-workspace/commit/ebb2eb2fe2ebfbb70530d16a983d396aa5829aa1) fix(persisted-scope): Prevent out-of-memory issues, fixes [#274](https://github.com/tauri-apps/plugins-workspace/pull/274) ([#328](https://github.com/tauri-apps/plugins-workspace/pull/328)) on 2023-04-26
