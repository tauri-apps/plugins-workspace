---
"updater": minor
"updater-js": minor
---

**Breaking change:** the `allowDowngrades` option was removed from the `check` command and is now read from the plugin configuration instead.

Previously any code running in the webview could pass `allowDowngrades: true` to `plugin:updater|check` and relax the version check from "the update must be newer" to "the update must be different", overriding the comparator the application had configured on the Rust side. The flag is now an application-level setting:

```json
{
  "plugins": {
    "updater": {
      "allowDowngrades": true
    }
  }
}
```

It defaults to `false`, and is ignored when the application provides its own `Builder::default_version_comparator`, which continues to take precedence.
