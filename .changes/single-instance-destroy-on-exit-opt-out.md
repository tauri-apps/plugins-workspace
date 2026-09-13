---
"log": minor:feat
"log-js": minor
---

Extend single-instance plugin with `destroy_on_exit` flag to allow consumers to opt-out out of automatically destroying the plugin on `tauri::RunEvent::Exit` as some consumers may prefer to defer releasing the single-instance lock until after application-specific cleanup has been performed.
