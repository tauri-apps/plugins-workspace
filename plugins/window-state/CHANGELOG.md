# Changelog

## \[0.1.1]

- Address a couple of issues with restoring positions:

- Fix restoring window positions correctly when the top-left corner of the window was outside of the monitor.

- Fix restore maximization state only maximized on main monitor.

- [70d9908](https://github.com/tauri-apps/plugins-workspace/commit/70d99086de3a58189d65c49954a3495972880725) fix(window-state): restore window position if the one of the window corners intersects with monitor ([#898](https://github.com/tauri-apps/plugins-workspace/pull/898)) on 2024-01-25
