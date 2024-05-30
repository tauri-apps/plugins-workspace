---
"shell": "patch"
---

Run `Command.execute()` JS api, asynchronously in the Rust side to avoid blocking main thread and causing the webview to freeze.
