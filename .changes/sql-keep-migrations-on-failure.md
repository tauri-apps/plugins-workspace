---
"sql": patch
"sql-js": patch
---

Fixed `load` dropping a database's migrations when one of them fails. A second `load` of the same database used to return successfully without running the remaining migrations, leaving the schema half migrated. It now runs them again and reports the error until they succeed.
