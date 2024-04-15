# Changelog

## \[1.0.5]

- `TrayLeft`, `TrayRight` and `TrayCenter` will now position the window according to the tray position relative to the monitor dimensions to prevent windows being displayed partially off-screen.
  - [3d27909](https://github.com/tauri-apps/plugins-workspace/commit/3d279094d44be78cdc5d1de3938f1414e13db6b0) fix(positioner): Prevent tray relative windows from being moved off-screen ([#291](https://github.com/tauri-apps/plugins-workspace/pull/291)) on 2023-09-27

## \[0.2.7]

- Update Tauri to v1.0.0
  - Bumped due to a bump in tauri-plugin-positioner.
  - [0bb73eb](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/0bb73eb20dae87f730c0b5f4cc08e6689e25fdba) Create tauri-v1.md on 2022-06-16

## \[0.2.6]

- Update Tauri to v1.0.0-rc.12
  - Bumped due to a bump in tauri-plugin-positioner.
  - [de6d76f](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/de6d76f3a96a68e88a7ac434d65df4dbc82af122) Create update-tauri.md on 2022-05-25

## \[0.2.5]

- Update deps
  - Bumped due to a bump in tauri-plugin-positioner.
  - [a8d3f73](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/a8d3f73b74829ef5d53d4fb028e59d09e8946399) Create chore-update-deps.md on 2022-05-18

## \[0.2.4]

- Update Tauri dependencies
  - [2095b6a](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/2095b6a4a4ab5590add099ddb2b1e8118e3496e4) add dep update changefile on 2022-02-14
  - [53d3a50](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/53d3a501776f16741124aa961f521b9d7798c878) publish new versions ([#42](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/pull/42)) on 2022-02-14
  - [9f32726](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/9f32726ede38bb9b2711f738a2f9fee7f0da2d73) Create update-deps.md on 2022-05-11

## \[0.2.3]

- **Breaking Change**: Uses the new Tauri plugin builder pattern. Use `tauri_plugin_positioner::init()` instead of `tauri_plugin_positioner::Positioner::default()`.
  - Bumped due to a bump in tauri-plugin-positioner.
  - [14837a8](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/14837a8d9cecdd6014867d4ef00fb98f21b2249d) refactor: use new builder pattern on 2022-02-26
  - [59874d8](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/59874d827471dfb889662fadc74fec1f2243b89e) fix typo on 2022-02-26

## \[0.2.2]

- Update README.md
  - Bumped due to a bump in tauri-plugin-positioner.
  - [92d6c3d](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/92d6c3dca00a6b3562682804a649c0023831ce2b) Create docs-update-readme.md on 2022-02-17

## \[0.2.1]

- Update `tauri` to `1.0.0-rc.1`, `serde` to `1.0.136` and `serde_json` to `1.0.79`.
  - [2095b6a](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/2095b6a4a4ab5590add099ddb2b1e8118e3496e4) add dep update changefile on 2022-02-14

## \[0.2.0]

- Add SystemTray relative positions.
  - [765b3ed](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/765b3ed90056d85ae88b0852b7107ff2b84a6c3a) Create feat-tray-pos.md on 2022-01-19

## \[0.1.0]

- Update package/crate metadata
  - [119d9c4](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/119d9c47639e1df16f5520a08f039bdb6f39532b) update metadata on 2021-11-19
  - [39e517c](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/39e517c145a4a901839ae9b46e296370ce6ababf) Update update-metadata.md on 2021-11-19
    data on 2021-11-19
  - [39e517c](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/39e517c145a4a901839ae9b46e296370ce6ababf) Update update-metadata.md on 2021-11-19
