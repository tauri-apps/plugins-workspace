---
global-shortcut: patch
global-shortcut-js: patch
---

Release the shortcuts lock before running shortcut handlers, so a handler that calls back into the plugin, or pumps messages that dispatch another shortcut event, no longer deadlocks the app.
