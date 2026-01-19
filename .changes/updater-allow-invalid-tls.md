---
"updater": minor
"updater-js": minor
---

Allow configuring the updater client to accept invalid TLS certificates and hostnames for internal/self-signed update servers. These options are available via the plugin config (`dangerousAcceptInvalidCerts`, `dangerousAcceptInvalidHostnames`) and via the `UpdaterBuilder` (`dangerous_accept_invalid_certs`, `dangerous_accept_invalid_hostnames`).
