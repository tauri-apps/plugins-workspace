To test the updater:

1. Comment out `verify_signature` inside `plugins/updater/src/updater.rs`
2. Run `pnpm tauri build --debug` to build the bundles
3. Copy the bundle you want to test to this folder (`examples/api/updater-test/`)
4. Run `pnpm tauri dev` and open the Updater tab to check

If the bundle you're testing doesn't exist in the `updater.json` yet, add it manually and welcome to open an PR

> The `updater.json` and debug bundles will be served by `vite`
