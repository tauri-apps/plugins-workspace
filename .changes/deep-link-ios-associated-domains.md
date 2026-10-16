---
"deep-link": patch
"deep-link-js": patch
---

The iOS associated domains entitlement is no longer overwritten on build: the plugin now only replaces its `applinks:` entries and keeps the other entries your app adds, such as `webcredentials:` for passkeys.
