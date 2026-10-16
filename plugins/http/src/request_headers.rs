// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The headers the plugin adds to webview requests: the default `User-Agent` and `Origin`, the
//! [`RequestHeaderRule`]s and the [`Builder::on_request`](crate::Builder::on_request) hook.

use std::{
    collections::{BTreeMap, HashSet},
    fmt,
};

use http::{HeaderMap, HeaderName, HeaderValue, Method, header};
use serde::{Deserialize, Deserializer};
use url::Url;
use urlpattern::{UrlPattern, UrlPatternMatchInput};

use crate::scope::parse_url_pattern;

const DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Headers that rules cannot set or remove: they describe the message framing or the target, or,
/// for `Cookie`, come from the cookie jar.
const DENIED_HEADERS: &[HeaderName] = &[
    header::HOST,
    header::CONTENT_LENGTH,
    header::TRANSFER_ENCODING,
    header::COOKIE,
];

pub(crate) type OnRequest = Box<dyn Fn(&RequestContext<'_>, &mut HeaderMap) + Send + Sync>;

/// When a [`RequestHeaderRule`] applies its headers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HeaderRuleMode {
    /// Applies a header only if the frontend did not set it. This replaces the `User-Agent` and
    /// `Origin` the plugin derives, but keeps the values the frontend sets explicitly.
    #[default]
    Default,
    /// Applies the headers unconditionally.
    Override,
}

/// Sets or removes request headers on webview requests to the URLs matching a pattern.
///
/// Rules are configured on the `requestHeaders` array of the [`Config`](crate::Config), or with
/// [`Builder::request_headers`](crate::Builder::request_headers). Every rule matching the request
/// URL applies, in order, so a later rule overrides an earlier one for the same header.
///
/// The `url` pattern uses the same syntax as the HTTP scope entries. It only decides which requests
/// get the headers: requests must still be allowed by the scope.
///
/// The rules apply again to every redirect the plugin follows, matched against the redirect
/// target, so the headers are never sent to a URL the rule does not match.
///
/// Rules can set headers the frontend is not allowed to send, such as `Origin`, but not `Host`,
/// `Content-Length`, `Transfer-Encoding` and `Cookie`. Their values are treated as sensitive: they
/// are left out of the `Debug` output of the rule and of the requests.
///
/// ```json
/// {
///   "url": "https://api.example.com/*",
///   "headers": { "Origin": "https://desktop.example.com", "X-Debug": null },
///   "mode": "override"
/// }
/// ```
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestHeaderRule {
    url: String,
    #[serde(deserialize_with = "deserialize_headers")]
    headers: Vec<(String, Option<String>)>,
    #[serde(default)]
    mode: HeaderRuleMode,
}

fn deserialize_headers<'de, D>(deserializer: D) -> Result<Vec<(String, Option<String>)>, D::Error>
where
    D: Deserializer<'de>,
{
    BTreeMap::<String, Option<String>>::deserialize(deserializer)
        .map(|map| map.into_iter().collect())
}

impl fmt::Debug for RequestHeaderRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // the values may be secrets, such as API keys
        let headers = self
            .headers
            .iter()
            .map(|(name, value)| (name, value.as_ref().map(|_| "Sensitive")))
            .collect::<Vec<_>>();
        f.debug_struct("RequestHeaderRule")
            .field("url", &self.url)
            .field("headers", &headers)
            .field("mode", &self.mode)
            .finish()
    }
}

impl RequestHeaderRule {
    /// Creates a rule for the URLs matching the given pattern, with no headers and the
    /// [`HeaderRuleMode::Default`] mode.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
            mode: HeaderRuleMode::Default,
        }
    }

    /// Sets the given header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), Some(value.into())));
        self
    }

    /// Removes the given header.
    pub fn remove_header(mut self, name: impl Into<String>) -> Self {
        self.headers.push((name.into(), None));
        self
    }

    /// Sets when the rule applies its headers.
    pub fn mode(mut self, mode: HeaderRuleMode) -> Self {
        self.mode = mode;
        self
    }
}

/// The request a [`Builder::on_request`](crate::Builder::on_request) hook is called for.
#[derive(Debug)]
pub struct RequestContext<'a> {
    url: &'a Url,
    method: &'a Method,
    webview_label: &'a str,
    webview_origin: Option<&'a str>,
    frontend_headers: &'a HashSet<HeaderName>,
}

impl<'a> RequestContext<'a> {
    pub(crate) fn new(
        url: &'a Url,
        method: &'a Method,
        webview_label: &'a str,
        webview_origin: Option<&'a str>,
        frontend_headers: &'a HashSet<HeaderName>,
    ) -> Self {
        Self {
            url,
            method,
            webview_label,
            webview_origin,
            frontend_headers,
        }
    }

    /// The URL of the request: the one requested by the frontend, or the target of a redirect the
    /// plugin is following.
    pub fn url(&self) -> &Url {
        self.url
    }

    /// The request method, which a redirect can change to `GET`.
    pub fn method(&self) -> &Method {
        self.method
    }

    /// The label of the webview that sent the request.
    pub fn webview_label(&self) -> &str {
        self.webview_label
    }

    /// The origin of the webview that sent the request, which the plugin sends as `Origin` by
    /// default: `tauri://localhost` or `http://tauri.localhost` in production, depending on the
    /// platform, and the dev server URL during development.
    pub fn webview_origin(&self) -> Option<&str> {
        self.webview_origin
    }

    /// Whether the frontend set the given header.
    ///
    /// Headers the frontend is not allowed to send, such as `Origin` without the `unsafe-headers`
    /// feature, are dropped before the hook runs, so they are never reported as set. Neither are
    /// the frontend headers a redirect dropped: `Authorization` on a redirect to another origin,
    /// and `Content-Type` when a redirect turns the request into a `GET`.
    pub fn frontend_set(&self, name: &HeaderName) -> bool {
        self.frontend_headers.contains(name)
    }
}

/// An invalid `userAgent` or `requestHeaders` value, reported when the plugin is set up.
#[derive(Debug)]
pub(crate) struct InvalidRequestHeaders(String);

impl fmt::Display for InvalidRequestHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HTTP plugin configuration: {}", self.0)
    }
}

impl std::error::Error for InvalidRequestHeaders {}

struct CompiledRule {
    url: UrlPattern,
    headers: Vec<(HeaderName, Option<HeaderValue>)>,
    mode: HeaderRuleMode,
}

impl CompiledRule {
    fn new(index: usize, rule: &RequestHeaderRule) -> Result<Self, InvalidRequestHeaders> {
        let error = |reason: String| {
            InvalidRequestHeaders(format!(
                "request header rule #{index} (`{}`): {reason}",
                rule.url
            ))
        };

        let url = parse_url_pattern(&rule.url)
            .map_err(|e| error(format!("`{}` is not a valid URL pattern: {e}", rule.url)))?;

        let mut headers: Vec<(HeaderName, Option<HeaderValue>)> = Vec::new();
        for (name, value) in &rule.headers {
            let name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| error(format!("`{name}` is not a valid header name: {e}")))?;
            if DENIED_HEADERS.contains(&name) {
                return Err(error(format!(
                    "the `{name}` header cannot be set by a rule"
                )));
            }
            if headers.iter().any(|(n, _)| *n == name) {
                return Err(error(format!(
                    "the `{name}` header is listed more than once"
                )));
            }
            let value = value
                .as_deref()
                .map(HeaderValue::from_str)
                .transpose()
                .map_err(|e| error(format!("invalid value for the `{name}` header: {e}")))?
                .map(|mut value| {
                    // keeps secrets such as API keys out of the request's `Debug` output
                    value.set_sensitive(true);
                    value
                });
            headers.push((name, value));
        }

        Ok(Self {
            url,
            headers,
            mode: rule.mode,
        })
    }

    fn matches(&self, url: &Url) -> bool {
        self.url
            .test(UrlPatternMatchInput::Url(url.clone()))
            .unwrap_or_default()
    }
}

/// The validated `userAgent`, `requestHeaders` and `on_request` hook.
pub(crate) struct RequestHeaders {
    user_agent: HeaderValue,
    rules: Vec<CompiledRule>,
    on_request: Option<OnRequest>,
}

impl RequestHeaders {
    pub(crate) fn new(
        user_agent: Option<&str>,
        rules: &[RequestHeaderRule],
        on_request: Option<OnRequest>,
    ) -> Result<Self, InvalidRequestHeaders> {
        let user_agent = HeaderValue::from_str(user_agent.unwrap_or(DEFAULT_USER_AGENT))
            .map_err(|e| InvalidRequestHeaders(format!("invalid user agent: {e}")))?;
        let rules = rules
            .iter()
            .enumerate()
            .map(|(index, rule)| CompiledRule::new(index, rule))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            user_agent,
            rules,
            on_request,
        })
    }

    /// Adds the plugin's default `User-Agent` and `Origin`, then applies the matching rules, then
    /// runs the hook.
    pub(crate) fn apply(
        &self,
        ctx: &RequestContext<'_>,
        headers: &mut HeaderMap,
    ) -> crate::Result<()> {
        if !headers.contains_key(header::USER_AGENT) {
            headers.insert(header::USER_AGENT, self.user_agent.clone());
        }

        // ensure we have an Origin header set
        if (cfg!(not(feature = "unsafe-headers")) || !headers.contains_key(header::ORIGIN))
            && let Some(origin) = ctx.webview_origin
        {
            headers.insert(header::ORIGIN, HeaderValue::from_str(origin)?);
        }

        // In case empty origin is passed, remove it. Some services do not like Origin header
        // so this way we can remove it in explicit way. The default behaviour is still to set it
        if cfg!(feature = "unsafe-headers")
            && headers.get(header::ORIGIN) == Some(&HeaderValue::from_static(""))
        {
            headers.remove(header::ORIGIN);
        };

        for rule in self.rules.iter().filter(|rule| rule.matches(ctx.url)) {
            for (name, value) in &rule.headers {
                if rule.mode == HeaderRuleMode::Default && ctx.frontend_set(name) {
                    continue;
                }
                match value {
                    Some(value) => headers.insert(name.clone(), value.clone()),
                    None => headers.remove(name),
                };
            }
        }

        if let Some(on_request) = &self.on_request {
            on_request(ctx, headers);
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    const WEBVIEW_ORIGIN: &str = "tauri://localhost";

    fn rules(json: &str) -> Vec<RequestHeaderRule> {
        serde_json::from_str(json).unwrap()
    }

    /// Applies the headers to a GET request from the frontend.
    pub(crate) fn apply(
        policy: &RequestHeaders,
        url: &str,
        frontend: &[(&str, &str)],
    ) -> HeaderMap {
        let url: Url = url.parse().unwrap();
        let mut headers = HeaderMap::new();
        for (name, value) in frontend {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        let frontend_headers = headers.keys().cloned().collect();
        let ctx = RequestContext::new(
            &url,
            &Method::GET,
            "main",
            Some(WEBVIEW_ORIGIN),
            &frontend_headers,
        );
        policy.apply(&ctx, &mut headers).unwrap();
        headers
    }

    pub(crate) fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
        headers.get(name).map(|v| v.to_str().unwrap())
    }

    #[test]
    fn plugin_defaults() {
        let policy = RequestHeaders::new(None, &[], None).unwrap();
        let headers = apply(&policy, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "user-agent"), Some(DEFAULT_USER_AGENT));
        assert_eq!(header(&headers, "origin"), Some(WEBVIEW_ORIGIN));

        let policy = RequestHeaders::new(Some("my-app/1.0"), &[], None).unwrap();
        let headers = apply(&policy, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "user-agent"), Some("my-app/1.0"));

        // the frontend's User-Agent takes precedence over the default
        let headers = apply(
            &policy,
            "https://api.example.com",
            &[("User-Agent", "web/1.0")],
        );
        assert_eq!(header(&headers, "user-agent"), Some("web/1.0"));
    }

    #[test]
    fn rules_apply_in_order() {
        let policy = RequestHeaders::new(
            None,
            &rules(
                r#"[
                    { "url": "https://*.example.com/*", "headers": { "Origin": "https://a.example.com", "X-A": "a" } },
                    { "url": "https://api.example.com/*", "headers": { "Origin": "https://b.example.com", "User-Agent": null } },
                    { "url": "https://other.test/*", "headers": { "X-Other": "1" } }
                ]"#,
            ),
            None,
        )
        .unwrap();

        let headers = apply(&policy, "https://api.example.com/v1/login", &[]);
        assert_eq!(header(&headers, "origin"), Some("https://b.example.com"));
        assert_eq!(header(&headers, "x-a"), Some("a"));
        assert_eq!(header(&headers, "user-agent"), None);
        assert_eq!(header(&headers, "x-other"), None);

        // only the first rule matches
        let headers = apply(&policy, "https://cdn.example.com/image.png", &[]);
        assert_eq!(header(&headers, "origin"), Some("https://a.example.com"));
        assert_eq!(header(&headers, "user-agent"), Some(DEFAULT_USER_AGENT));

        // no rule matches, so the plugin defaults apply
        let headers = apply(&policy, "https://unknown.test", &[]);
        assert_eq!(header(&headers, "origin"), Some(WEBVIEW_ORIGIN));
        assert_eq!(header(&headers, "x-a"), None);
    }

    #[test]
    fn default_mode_keeps_frontend_headers() {
        let policy = RequestHeaders::new(
            None,
            &rules(
                r#"[
                    { "url": "https://api.example.com", "headers": { "User-Agent": "rule/1.0", "X-Client": "rule" } },
                    { "url": "https://override.example.com", "headers": { "User-Agent": "rule/1.0", "X-Client": null }, "mode": "override" }
                ]"#,
            ),
            None,
        )
        .unwrap();

        let frontend = [("User-Agent", "web/1.0"), ("X-Client", "web")];

        let headers = apply(&policy, "https://api.example.com", &frontend);
        assert_eq!(header(&headers, "user-agent"), Some("web/1.0"));
        assert_eq!(header(&headers, "x-client"), Some("web"));

        // the rule still replaces the plugin's default
        let headers = apply(&policy, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "user-agent"), Some("rule/1.0"));

        let headers = apply(&policy, "https://override.example.com", &frontend);
        assert_eq!(header(&headers, "user-agent"), Some("rule/1.0"));
        assert_eq!(header(&headers, "x-client"), None);
    }

    #[test]
    fn hook_runs_last() {
        let policy = RequestHeaders::new(
            Some("my-app/1.0"),
            &rules(
                r#"[{ "url": "https://*", "headers": { "Origin": "https://rule.example.com" } }]"#,
            ),
            Some(Box::new(|ctx, headers| {
                assert_eq!(ctx.webview_label(), "main");
                assert_eq!(ctx.webview_origin(), Some(WEBVIEW_ORIGIN));
                assert_eq!(*ctx.method(), Method::GET);
                assert!(ctx.frontend_set(&header::ACCEPT));
                assert!(!ctx.frontend_set(&header::USER_AGENT));

                if ctx.url().host_str() == Some("api.example.com") {
                    headers.insert(
                        header::ORIGIN,
                        HeaderValue::from_static("https://hook.example.com"),
                    );
                    headers.remove(header::USER_AGENT);
                }
            })),
        )
        .unwrap();

        let headers = apply(&policy, "https://api.example.com", &[("Accept", "*/*")]);
        assert_eq!(header(&headers, "origin"), Some("https://hook.example.com"));
        assert_eq!(header(&headers, "user-agent"), None);

        let headers = apply(&policy, "https://cdn.example.com", &[("Accept", "*/*")]);
        assert_eq!(header(&headers, "origin"), Some("https://rule.example.com"));
        assert_eq!(header(&headers, "user-agent"), Some("my-app/1.0"));
    }

    #[test]
    fn invalid_configuration_is_rejected() {
        let invalid = [
            r#"[{ "url": "https://*", "headers": { "Cookie": "a=b" } }]"#,
            r#"[{ "url": "https://*", "headers": { "host": null } }]"#,
            r#"[{ "url": "https://*", "headers": { "Content-Length": "0" } }]"#,
            r#"[{ "url": "https://*", "headers": { "Transfer-Encoding": "chunked" } }]"#,
            r#"[{ "url": "https://*", "headers": { "Bad Name": "x" } }]"#,
            r#"[{ "url": "https://*", "headers": { "X-Value": "line\nbreak" } }]"#,
            r#"[{ "url": "https://*", "headers": { "Origin": "a", "origin": "b" } }]"#,
            r#"[{ "url": "https://[", "headers": {} }]"#,
        ];
        for json in invalid {
            assert!(
                RequestHeaders::new(None, &rules(json), None).is_err(),
                "{json} should be rejected"
            );
        }

        assert!(RequestHeaders::new(Some("line\nbreak"), &[], None).is_err());

        // unknown fields and modes fail to deserialize
        assert!(
            serde_json::from_str::<Vec<RequestHeaderRule>>(
                r#"[{ "url": "https://*", "header": {} }]"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<Vec<RequestHeaderRule>>(
                r#"[{ "url": "https://*", "headers": {}, "mode": "replace" }]"#
            )
            .is_err()
        );
    }

    #[test]
    fn null_removes_plugin_defaults_but_not_frontend_headers() {
        let policy = RequestHeaders::new(
            None,
            &rules(
                r#"[{ "url": "https://api.example.com", "headers": { "Origin": null, "User-Agent": null, "X-Client": null } }]"#,
            ),
            None,
        )
        .unwrap();

        let headers = apply(&policy, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "origin"), None);
        assert_eq!(header(&headers, "user-agent"), None);

        let headers = apply(
            &policy,
            "https://api.example.com",
            &[("User-Agent", "web/1.0"), ("X-Client", "web")],
        );
        assert_eq!(header(&headers, "user-agent"), Some("web/1.0"));
        assert_eq!(header(&headers, "x-client"), Some("web"));
    }

    #[test]
    fn rules_match_like_scope_entries() {
        let policy = RequestHeaders::new(
            None,
            &rules(
                r#"[
                    { "url": "http://localhost:*", "headers": { "X-Local": "1" } },
                    { "url": "https://api.example.com/v1/*", "headers": { "X-V1": "1" } }
                ]"#,
            ),
            None,
        )
        .unwrap();

        for url in ["http://localhost:5173", "http://localhost:8080/path?q=1"] {
            assert_eq!(
                header(&apply(&policy, url, &[]), "x-local"),
                Some("1"),
                "{url}"
            );
        }
        for url in ["https://localhost:5173", "http://127.0.0.1:5173"] {
            assert_eq!(header(&apply(&policy, url, &[]), "x-local"), None, "{url}");
        }

        assert_eq!(
            header(
                &apply(&policy, "https://api.example.com/v1/users?id=1", &[]),
                "x-v1"
            ),
            Some("1")
        );
        assert_eq!(
            header(
                &apply(&policy, "https://api.example.com/v2/users", &[]),
                "x-v1"
            ),
            None
        );
        assert_eq!(
            header(
                &apply(&policy, "https://api.example.com.evil.test/v1/users", &[]),
                "x-v1"
            ),
            None
        );
    }

    #[test]
    fn rule_values_are_sensitive() {
        let rule = RequestHeaderRule::new("https://api.example.com/*")
            .header("X-Api-Key", "s3cret")
            .remove_header("X-Debug");
        let debug = format!("{rule:?}");
        assert!(debug.contains("X-Api-Key"), "{debug}");
        assert!(!debug.contains("s3cret"), "{debug}");

        let policy = RequestHeaders::new(None, &[rule], None).unwrap();
        let headers = apply(&policy, "https://api.example.com/v1", &[]);
        assert!(headers.get("x-api-key").unwrap().is_sensitive());
        assert!(!format!("{headers:?}").contains("s3cret"));
    }

    #[test]
    fn rules_built_in_rust() {
        let rule = RequestHeaderRule::new("https://api.example.com/*")
            .header("Origin", "https://desktop.example.com")
            .remove_header("User-Agent")
            .mode(HeaderRuleMode::Override);
        let policy = RequestHeaders::new(None, &[rule], None).unwrap();

        let headers = apply(
            &policy,
            "https://api.example.com/v1",
            &[("User-Agent", "web/1.0")],
        );
        assert_eq!(
            header(&headers, "origin"),
            Some("https://desktop.example.com")
        );
        assert_eq!(header(&headers, "user-agent"), None);
    }
}
