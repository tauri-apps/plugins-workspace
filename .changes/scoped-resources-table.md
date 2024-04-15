---
"fs": "patch"
"http": "patch"
"updater": "patch"
"clipboard-manager": "patch"
---

Internally use the webview scoped resources table instead of the app one, so other webviews can't access other webviews resources.
