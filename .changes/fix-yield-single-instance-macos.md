---
"single-instance": minor-fix
"single-instance": minor-fix
---
Fix blocked thread on the single-instance plugin for MacOS: replace standard `UnixListener` with `tokio::net::UnixListener`, so the task can yield.
