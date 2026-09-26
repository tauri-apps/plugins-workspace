---
"barcode-scanner": patch
"biometric": patch
"clipboard-manager": patch
"deep-link": patch
"dialog": patch
"fs": patch
"geolocation": patch
"haptics": patch
"nfc": patch
"notification": patch
"opener": patch
"shell": patch
---

Migrate the Android Gradle scripts from the deprecated `kotlinOptions` DSL to `compilerOptions`, which Kotlin Gradle Plugin 2.3+ requires, to match the Tauri 2.12 Android library and templates.
