# Changelog

## \[2.0.0-alpha.2]

- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.

### Dependencies

- Upgraded to `fs@2.0.0-alpha.2`

## \[2.0.0-alpha.1]

- [`d5a7c77`](https://github.com/tauri-apps/plugins-workspace/commit/d5a7c77a8d0e7912a6b07b22ed329004edd6e80b)([#545](https://github.com/tauri-apps/plugins-workspace/pull/545)) Fixes docs.rs build by enabling the `tauri/dox` feature flag.
- [`aba07c2`](https://github.com/tauri-apps/plugins-workspace/commit/aba07c27b887c1cc54026024227cb3f74c91e21a)([#468](https://github.com/tauri-apps/plugins-workspace/pull/468)) Split up fs and asset scopes. **This will reset the asset protocol scope once!**
- [`aba07c2`](https://github.com/tauri-apps/plugins-workspace/commit/aba07c27b887c1cc54026024227cb3f74c91e21a)([#468](https://github.com/tauri-apps/plugins-workspace/pull/468)) Fix usage of directory patterns by removing glob asterisks at the end before allowing/forbidding them. This was causing them to be escaped, and so undesirable paths were allowed/forbidden while polluting the `.persisted-scope` file.
- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

### Dependencies

- Upgraded to `fs@2.0.0-alpha.1`

## \[2.0.0-alpha.0]

- [`ebb2eb2`](https://github.com/tauri-apps/plugins-workspace/commit/ebb2eb2fe2ebfbb70530d16a983d396aa5829aa1)([#274](https://github.com/tauri-apps/plugins-workspace/pull/274)) Recursively unescape saved patterns before allowing/forbidding them. This effectively prevents `.persisted-scope` files from blowing up, which caused Out-Of-Memory issues, while automatically fixing existing broken files seamlessly.
- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!

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
