---
"persisted-scope": patch
---

Fixing directories' usage by removing glob patterns asterisks at the end before allowing/forbidding them.

This was causing them to be escaped, and so undesirable paths were allowed/forbidden.

Polluting the `.persisted_scope` at the same time.