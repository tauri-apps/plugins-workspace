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

As there are only the `scan` and `cancel` commands exposed to the frontend,
there is no additional risk or exposure of additional information.
Only barcodes are passed and no raw camera access is used, which means no images are available to the frontend.

The application is only usable on iOS and Android and therefore the specific mobile operating system security boundaries need to be considered.

### Security Assumptions

- The QR code parsing into a link/text is trusted and correctly handled by the mobile operating system
- The link itself is untrusted and additional validation/sanitization needs to be handled by the app developer
- The camera is not passing images to the app
- The camera permission is granted at first use by the user and can be revoked at any time
- The Android manifest also states that the camera permission is required

### Threats

#### Silent Interaction

##### When is it possible?

The following threat is either caused by a malicious developer, which has further implications and should be considered as a full compromise of an application or system, or by
compromise of the application frontend. In the second case there are several impact minimization methods (e.g. the CSP) and if all of these fail the possible risk could occur.
Therefore it is unlikely to occur in most cases but should be considered when using this plugin.

##### What is possible?

The camera has two modes. The first one is where the user can see the background camera image and no further interaction is possible.
The second mode allows the developer to assist the user and add a transparent overlay to the image, providing hints or additional information (like a link preview).
The overlay could be made non-transparent by the application frontend and as long as the app is open (and in some cases) it could read QR codes in range of the camera lense.


#### Out Of Scope

- Exploits in the operating system QR code parsing functionality
- Exploits based on the string passed to the application using this plugin
- Continous camera/QR scan usage even when application is in background

## Best Practices

There is no additional exposure aside from reading barcodes in the webview and there are no specific best practices for secure usage.
