---
"updater": patch:bug
"updater-js": patch:bug
---

On Linux, `check()` no longer sets `SSL_CERT_FILE` and `SSL_CERT_DIR` to Debian paths. The override pointed rustls to files that do not exist on distros with a different layout, such as ALT Linux, and left every TLS client in the app without root certificates.
