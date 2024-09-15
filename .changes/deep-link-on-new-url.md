---
"deep-link": patch
---

Added `DeepLink::on_open_url` function to match the JavaScript API implementation,
which wraps the `deep-link://new-url` event and also send the current deep link if there's any.
