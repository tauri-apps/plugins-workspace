use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub scope: HttpAllowlistScope,
}

/// HTTP API scope definition.
/// It is a list of URLs that can be accessed by the webview when using the HTTP APIs.
/// The scoped URL is matched against the request URL using a glob pattern.
///
/// Examples:
/// - "https://**": allows all HTTPS urls
/// - "https://*.github.com/tauri-apps/tauri": allows any subdomain of "github.com" with the "tauri-apps/api" path
/// - "https://myapi.service.com/users/*": allows access to any URLs that begins with "https://myapi.service.com/users/"
#[allow(rustdoc::bare_urls)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize)]
pub struct HttpAllowlistScope(pub Vec<Url>);
