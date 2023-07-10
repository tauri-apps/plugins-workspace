# Changelog

## \[0.1.3]

- Split up fs and asset scopes. **This will reset the asset protocol scope once!**
  - [ad30286](https://github.com/tauri-apps/plugins-workspace/commit/ad3028646c96ed213a2f483823ffdc3c17b5fc1e) fix(persisted-scope): separately save asset protocol patterns ([#459](https://github.com/tauri-apps/plugins-workspace/pull/459)) on 2023-07-10

## \[0.1.2]

- Fix usage of directory patterns by removing glob asterisks at the end before allowing/forbidding them. This was causing them to be escaped, and so undesirable paths were allowed/forbidden while polluting the `.persisted_scope` file.
  - [9174b80](https://github.com/tauri-apps/plugins-workspace/commit/9174b808dc37154999c119fcc3f31258a9c5a3fb) \[persisted scope] fix: handle recursive directory correctly ([#455](https://github.com/tauri-apps/plugins-workspace/pull/455)) on 2023-06-29

## \[0.1.1]

- The MSRV was raised to 1.64!
- The plugin now recursively unescapes saved patterns before allowing/forbidding them. This effectively prevents `.persisted-scope` files from blowing up, which caused Out-Of-Memory issues, while automatically fixing existing broken files seamlessly.
  - [ebb2eb2](https://github.com/tauri-apps/plugins-workspace/commit/ebb2eb2fe2ebfbb70530d16a983d396aa5829aa1) fix(persisted-scope): Prevent out-of-memory issues, fixes [#274](https://github.com/tauri-apps/plugins-workspace/pull/274) ([#328](https://github.com/tauri-apps/plugins-workspace/pull/328)) on 2023-04-26
