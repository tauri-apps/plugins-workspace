---
"updater": patch
"updater-js": patch
---

fix: preserve file extension of updater package, otherwise users may get confused when presented with a sudo dialog suggesting to install a file with the extension `.rpm` using `dpkg -i`
