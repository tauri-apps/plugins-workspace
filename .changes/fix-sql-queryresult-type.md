---
sql-js: patch
---

Fixed the QueryResult typing by marking `lastInsertId` as optional to reflect postgres-only changes made in the 2.0.0 release.
