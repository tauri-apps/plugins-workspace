---
"barcode-scanner": patch
"barcode-scanner-js": patch
"biometric": patch
"biometric-js": patch
"clipboard-manager": patch
"clipboard-manager-js": patch
"deep-link": patch
"deep-link-js": patch
"dialog": patch
"dialog-js": patch
"fs": patch
"fs-js": patch
"geolocation": patch
"geolocation-js": patch
"haptics": patch
"haptics-js": patch
"nfc": patch
"nfc-js": patch
"notification": patch
"notification-js": patch
"opener": patch
"opener-js": patch
"shell": patch
"shell-js": patch
---

Migrate the Android Gradle scripts from the deprecated `kotlinOptions` DSL to `compilerOptions`, which is accepted by both Kotlin Gradle Plugin 1.9.x and 2.x. This lets projects move to Kotlin 2.x without hitting the hard error that 2.3+ raises on the old DSL.
