---
notification: minor
notification-js: minor
---

- Add Rust builders for mobile notification action types.
- Fix Android action-group entries being stored under the action type ID instead of sequential indices, causing later actions to overwrite earlier ones.
