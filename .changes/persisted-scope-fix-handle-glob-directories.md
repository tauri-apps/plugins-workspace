---
"persisted-scope": patch
---

Fix usage of directory patterns by removing glob asterisks at the end before allowing/forbidding them.

This was causing them to be escaped, and so undesirable paths were allowed/forbidden while polluting the `.persisted_scope` file.
