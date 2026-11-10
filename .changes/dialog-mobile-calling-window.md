---
"dialog": minor
"dialog-js": minor
---

On Android and iOS, dialogs opened through the JavaScript `open`, `save` and `message` commands now use the calling webview's activity or view controller. If that window is closed before the dialog can be shown or while it is open, the call fails with an error message that contains `ORIGIN_UNAVAILABLE` (`[ORIGIN_UNAVAILABLE] - ...`) instead of showing the dialog on another window or never returning.

On Android, file and save dialogs with an origin fail with an error message containing `RESULT_PENDING` if that activity already has a file or save dialog pending, instead of leaving the first call without an answer. On iOS, a dialog request waits for an ongoing modal dismissal to finish. If a modal remains open, including a fullscreen modal, or the presenter is otherwise transitioning, the request fails with `RESULT_PENDING`. The request is not reported as a cancellation.

Added `FileDialogBuilder::set_origin` and `MessageDialogBuilder::origin` on mobile, so Rust callers can choose a dialog's origin. Without an origin, the existing behavior is preserved: iOS uses the controller of the most recently registered webview; Android message dialogs use the activity the plugin was created with, while file and save dialogs use the plugin manager's oldest surviving activity.

Cancelling a file dialog still returns `null`, and other failures keep their previous result: `null` for file dialogs, and `Cancel` for message dialogs. Requires the `PluginHandle::run_mobile_plugin_with_webview` API from `tauri`.
