---
persisted-scope: patch
---

Recursively unescape saved patterns before allowing/forbidding them. This effectively prevents `.persisted-scope` files from blowing up, which caused Out-Of-Memory issues, while automatically fixing existing broken files seamlessly.
