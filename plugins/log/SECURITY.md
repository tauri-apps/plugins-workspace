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

### Security Assumptions

- The log file interpreting applications are hardened, as input is not sanitized
- No log events in the rust core are leaked to the frontend unless explicitly configured to output to the `TargetKind::Webview` component
- The log events generated in the frontend can be accessed from everywhere in the frontend
- There is no secret censoring inbuilt and developers need to take care of what they log in their application

### Threats

#### Secret Leakage

One possible threat you need to consider when using this plugin is that secrets
in logs can theoretically be leaked when the application's frontend gets compromised.

For this threat to be possible all of the following requirements need to be fulfilled:
- `TargetKind::Webview` enabled OR secrets stem from frontend logs
- Frontend application is compromised via something like XSS (cross-site-scripting) OR logs are directly exposed
- Logs contain secrets or sensitive information

If these requirements are not met, the leakage should not be possible.

#### Out Of Scope

- Any exploits on the log viewer/file viewer accessing the logs

## Best Practices

Do not log secrets or sensitive values in your logging and ensure that the upstream crates are not leaking such values in their logging events.
Ensure that logs are sanitized or trusted before opening them with third party tools.
