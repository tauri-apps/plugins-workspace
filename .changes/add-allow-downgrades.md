---
"updater": minor
"updater-js": minor
---

Add allowDowngrades parameter to check command

Added a new optional `allowDowngrades` parameter to the JavaScript check command that allows the updater to consider versions that are lower than the current version as valid updates. When enabled, the version comparator will accept any version that is different from the current version, effectively allowing downgrades.
