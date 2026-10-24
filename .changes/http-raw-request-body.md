---
"http": minor:feat
"http-js": minor:feat
---

`fetch` sends a request body as the raw IPC payload instead of a JSON array with one number per byte, which the webview built on its main thread. The request options travel in an IPC header. The `fetch` command still accepts the previous `{ clientConfig }` arguments. Android has no raw IPC payloads, so there the body still arrives as a number array.
