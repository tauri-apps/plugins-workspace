---
websocket: patch:bug
websocket-js: patch:bug
---

The WebSocket plugin will now install the default crypto provider if needed, preventing panics on WSS connections.
