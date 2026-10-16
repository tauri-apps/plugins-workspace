---
http: minor
http-js: minor
---

Added the `requestHeaders` configuration option, which sets or removes headers, including `Origin`, on requests to the URLs matching a pattern. By default a rule only replaces headers the frontend did not set; use `"mode": "override"` to always apply it.
