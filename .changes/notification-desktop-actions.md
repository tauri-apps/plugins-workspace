---
"notification": minor:feat
"notification-js": minor:feat
---

Report actions on desktop: a click on a notification (`tap`) or on one of the actions of its action type reaches `onAction` and the new Rust `Notification::on_action`, which also works on mobile. `registerActionTypes`, `removeActive` (Linux and the BSDs) and the listener commands no longer fail with "command not found" on desktop.
