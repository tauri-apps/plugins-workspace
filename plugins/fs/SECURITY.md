# Security Policy

**Do not report security vulnerabilities through public GitHub issues.**

**Please use the [Private Vulnerability Disclosure](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability#privately-reporting-a-security-vulnerability) feature of GitHub.**

Include as much of the following information:

- Type of issue (e.g. improper input parsing, privilege escalation, etc.)
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- The distribution affected or used to help us with reproduction of the issue
- Step-by-step instructions to reproduce the issue
- Ideally a reproduction repository
- Impact of the issue, including how an attacker might exploit the issue

We prefer to receive reports in English.

## Contact

Please disclose a vulnerability or security relevant issue here: [https://github.com/tauri-apps/plugins-workspace/security/advisories/new](https://github.com/tauri-apps/plugins-workspace/security/advisories/new).

Alternatively, you can also contact us by email via [security@tauri.app](mailto:security@tauri.app).

## Threat Model

This plugin possibly allows access to the full filesystem available to the application process.
Depending on the operating system the access is already confined (android/ios) to only certain locations.
In other operating systems like Linux/MacOS/Windows it depends on the installation and packaging method but in most cases full
access is granted.

To prevent exposure of sensitive locations and data this plugin can be scoped to only allow certain base directories
or only access to specific files or subdirectories.
This scoping effectively affects only calls made from the webviews/frontend code and calls made from rust can always circumvent
the restrictions imposed by the scope.

The scope is defined at compile time in the used permissions but the user or application developer can grant or revoke access to specific files or folders at runtime by modifying the scope state through the runtime authority, if configured during plugin initialization.

### Security Assumptions

- The filesystem access is limited by user permissions
- The operating system filesystem access confinment works as documented
- The scoping mechanism of the Tauri `fs` commands work as intended and has no bypasses
- The user or application developer can grant or revoke access to specific files at runtime by modifying the scope

#### Out Of Scope

- Exploits in underlying filesystems
- Exploits in the underlying rust `std::fs` library
