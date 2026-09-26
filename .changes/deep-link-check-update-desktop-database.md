---
deep-link: patch
deep-link-js: patch
---

On Linux, skip the `update-desktop-database` call during registration when the command is not installed (e.g. some KDE setups without `desktop-file-utils`) instead of failing.
