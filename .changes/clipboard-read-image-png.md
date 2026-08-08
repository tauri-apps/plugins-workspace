---
"clipboard-manager": minor:feat
"clipboard-manager-js": minor:feat
---

Added `readImagePNG` to read the clipboard image as PNG-encoded bytes instead of a raw decoded RGBA buffer, which is significantly smaller to transfer for large images (e.g. UHD screenshots).
