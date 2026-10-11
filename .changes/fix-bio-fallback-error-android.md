---
"biometric": patch:bug
"biometric-js": patch:bug
---

Fix biometric plugin reporting an inconsistent and incorrect error on Android when no device passcode was set and `allowDeviceCredential` is enabled. Android now rejects with the `passcodeNotSet` error code like iOS does; it previously used the undocumented `noDeviceCredential` code, so code matching that string must switch to `passcodeNotSet`.
