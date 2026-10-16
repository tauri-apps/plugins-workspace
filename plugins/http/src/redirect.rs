// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Follows redirects for [`fetch`](crate::commands::fetch), computing the headers of every hop.

use std::collections::HashSet;

use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};

use crate::{Error, Result, scope::Scope};

/// Matches the default [`reqwest`] redirect policy, used when the frontend does not
/// configure `maxRedirections`.
pub(crate) const DEFAULT_MAX_REDIRECTIONS: usize = 10;

/// How [`send`] follows redirects.
pub(crate) struct Redirects {
    /// When [`Some`], **every** hop is checked against the scope. Validating only the URL the
    /// frontend asked for is not enough: a server on an allowed origin can answer with a redirect
    /// to any other origin (an open redirect, or a server the attacker controls), and following it
    /// would give the webview access to a URL the scope denies.
    ///
    /// That check is opt-in through the `scopeRedirects` configuration because it breaks
    /// applications that rely on being redirected outside of their scope.
    // TODO(v3): always check the scope and take it by value instead of `Option`
    pub(crate) scope: Option<Scope>,
    /// How many redirects are followed before failing. `0` returns the redirect response instead.
    pub(crate) max: usize,
    /// Whether a redirected request carries the previous URL as `Referer`, as
    /// [`reqwest::ClientBuilder::referer`] does. The client does not follow redirects, so its own
    /// setting has no effect.
    pub(crate) referer: bool,
}

/// Sends `request`, following redirects the way [`reqwest`]'s default policy does, except that the
/// plugin follows them itself so the headers of every hop are computed for its URL.
///
/// `headers` holds the frontend's headers and the ones derived from the request; `add_headers`
/// adds the user agent, origin, request header rules and `on_request` hook for a hop, given its URL,
/// method and the names of the frontend headers it still carries. That way nothing added for one
/// URL is sent to a redirect target it was not meant for. The client must not follow redirects.
///
/// The headers `request` already has, such as the `Authorization` [`reqwest`] derives from the
/// credentials of the URL, count as the frontend's unless `headers` sets them too.
pub(crate) async fn send<F>(
    client: reqwest::Client,
    mut request: reqwest::Request,
    mut headers: HeaderMap,
    mut frontend_headers: HashSet<HeaderName>,
    redirects: Redirects,
    add_headers: F,
) -> Result<reqwest::Response>
where
    F: Fn(&url::Url, &Method, &HashSet<HeaderName>, &mut HeaderMap) -> Result<()>,
{
    let request_headers = std::mem::take(request.headers_mut());
    for name in request_headers.keys() {
        if !headers.contains_key(name) {
            for value in request_headers.get_all(name) {
                headers.append(name.clone(), value.clone());
            }
            frontend_headers.insert(name.clone());
        }
    }

    let mut redirections = 0;
    let mut referer = None;

    loop {
        // kept to send the request again if the response is a redirect; the body is in memory
        let next = request.try_clone();

        let mut hop_headers = headers.clone();
        if let Some(referer) = referer.take() {
            hop_headers.insert(header::REFERER, referer);
        }
        add_headers(
            request.url(),
            request.method(),
            &frontend_headers,
            &mut hop_headers,
        )?;
        *request.headers_mut() = hop_headers;

        #[cfg(feature = "tracing")]
        tracing::trace!("{:?}", request);

        let response = client.execute(request).await?;

        let Some(location) = redirect_location(&response) else {
            return Ok(response);
        };
        let Some(mut next) = next.filter(|_| redirects.max > 0) else {
            return Ok(response);
        };

        redirections += 1;
        if redirections > redirects.max {
            return Err(Error::TooManyRedirects(location));
        }
        if let Some(scope) = &redirects.scope
            && !scope.is_allowed(&location)
        {
            return Err(Error::UrlNotAllowed(location));
        }

        // https://fetch.spec.whatwg.org/#http-redirect-fetch step 12
        let drop_payload = match response.status() {
            StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => next.method() == Method::POST,
            StatusCode::SEE_OTHER => true,
            _ => false,
        };
        if drop_payload {
            if next.method() != Method::HEAD {
                *next.method_mut() = Method::GET;
            }
            *next.body_mut() = None;
            for name in [
                header::CONTENT_TYPE,
                header::CONTENT_LENGTH,
                header::CONTENT_ENCODING,
                header::CONTENT_LANGUAGE,
                header::CONTENT_LOCATION,
                header::TRANSFER_ENCODING,
            ] {
                headers.remove(&name);
                frontend_headers.remove(&name);
            }
        }

        // the credentials were meant for the origin the frontend requested: once the chain leaves
        // it, they are not sent again
        if next.url().origin() != location.origin() {
            for name in [
                header::AUTHORIZATION,
                header::COOKIE,
                HeaderName::from_static("cookie2"),
                header::PROXY_AUTHORIZATION,
                header::WWW_AUTHENTICATE,
            ] {
                headers.remove(&name);
                frontend_headers.remove(&name);
            }
        }

        if redirects.referer {
            referer = make_referer(&location, next.url());
        }
        *next.url_mut() = location;
        request = next;
    }
}

/// The URL a response redirects to, if it is a redirect the plugin follows.
fn redirect_location(response: &reqwest::Response) -> Option<url::Url> {
    if !matches!(
        response.status(),
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    ) {
        return None;
    }
    let location = response.headers().get(header::LOCATION)?;
    let location = std::str::from_utf8(location.as_bytes()).ok()?;
    response.url().join(location).ok()
}

/// The `Referer` reqwest sends on redirects: the previous URL without credentials and fragment,
/// and nothing when going from `https` to `http`.
fn make_referer(next: &url::Url, previous: &url::Url) -> Option<HeaderValue> {
    if next.scheme() == "http" && previous.scheme() == "https" {
        return None;
    }

    let mut referer = previous.clone();
    let _ = referer.set_username("");
    let _ = referer.set_password(None);
    referer.set_fragment(None);
    referer.as_str().parse().ok()
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        sync::Arc,
    };

    use reqwest::redirect::Policy;

    use super::*;
    use crate::{RequestContext, RequestHeaderRule, request_headers::RequestHeaders};

    /// Test server with an open redirect:
    ///
    /// - `/redirect-external` answers a 302 to `http://127.0.0.1:{port}/secret`, which is a
    ///   different origin as far as the scope is concerned - the tests allow `localhost`;
    /// - `/redirect/{n}` answers a 302 to `/redirect/{n-1}` on `localhost`, and `/redirect/0`
    ///   answers a 302 to `/target`, so the whole chain stays in scope;
    /// - `/r/{status}/{host}/{path}` answers a `{status}` redirect to `http://{host}:{port}/{path}`,
    ///   or to the relative `/{path}` when `{host}` is `-`;
    /// - anything else answers a 200 with the method, path, headers and body it received as JSON,
    ///   so a bypass shows up as a successful response.
    fn spawn_server() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();

                // the whole request must be consumed, otherwise closing the connection
                // with unread data queued resets it before the client reads the response
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let mut headers = serde_json::Map::new();
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if matches!(header.as_str(), "" | "\r\n" | "\n") {
                        break;
                    }
                    if let Some((name, value)) = header.split_once(':') {
                        headers.insert(name.trim().to_ascii_lowercase(), value.trim().into());
                    }
                }
                let length = headers
                    .get("content-length")
                    .and_then(|value| value.as_str()?.parse().ok())
                    .unwrap_or(0);
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();

                let mut parts = request_line.split(' ');
                let method = parts.next().unwrap_or_default();
                let path = parts.next().unwrap_or("/");
                let redirect = match path.strip_prefix("/redirect/") {
                    Some("0") => Some(("302", format!("http://localhost:{port}/target"))),
                    Some(remaining) => Some((
                        "302",
                        format!(
                            "http://localhost:{port}/redirect/{}",
                            remaining.parse::<usize>().unwrap() - 1
                        ),
                    )),
                    None if path == "/redirect-external" => {
                        Some(("302", format!("http://127.0.0.1:{port}/secret")))
                    }
                    None => path.strip_prefix("/r/").map(|rest| {
                        let mut parts = rest.splitn(3, '/');
                        let status = parts.next().unwrap();
                        let host = parts.next().unwrap();
                        let path = parts.next().unwrap_or_default();
                        let location = if host == "-" {
                            format!("/{path}")
                        } else {
                            format!("http://{host}:{port}/{path}")
                        };
                        (status, location)
                    }),
                };

                let response = match redirect {
                    Some((status, location)) => format!(
                        "HTTP/1.1 {status} Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    ),
                    None => {
                        let echo = serde_json::json!({
                            "method": method,
                            "path": path,
                            "headers": headers,
                            "body": String::from_utf8_lossy(&body),
                        })
                        .to_string();
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{echo}",
                            echo.len()
                        )
                    }
                };

                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        port
    }

    /// Sends a request the way `fetch` does, with the given frontend headers and header rules.
    fn fetch(
        method: Method,
        url: &str,
        body: Option<&str>,
        frontend: &[(&str, &str)],
        redirects: Redirects,
        rules: &[RequestHeaderRule],
    ) -> Result<reqwest::Response> {
        let client = reqwest::ClientBuilder::new()
            .redirect(Policy::none())
            .build()
            .unwrap();

        let mut headers = HeaderMap::new();
        for (name, value) in frontend {
            headers.append(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        let frontend_headers = headers.keys().cloned().collect();

        let mut request = client.request(method, url);
        if let Some(body) = body {
            request = request.body(body.to_string());
        }
        let request = request.build().unwrap();

        let request_headers = RequestHeaders::new(None, rules, None).unwrap();
        let add_headers = move |url: &url::Url,
                                method: &Method,
                                frontend_headers: &HashSet<HeaderName>,
                                headers: &mut HeaderMap| {
            let ctx = RequestContext::new(
                url,
                method,
                "main",
                Some("tauri://localhost"),
                frontend_headers,
            );
            request_headers.apply(&ctx, headers)
        };

        tauri::async_runtime::block_on(send(
            client,
            request,
            headers,
            frontend_headers,
            redirects,
            add_headers,
        ))
    }

    fn get(
        url: &str,
        scope: Option<Scope>,
        max_redirections: Option<usize>,
    ) -> Result<reqwest::Response> {
        let redirects = Redirects {
            scope,
            max: max_redirections.unwrap_or(DEFAULT_MAX_REDIRECTIONS),
            referer: true,
        };
        fetch(Method::GET, url, None, &[], redirects, &[])
    }

    /// Follows redirects without a scope, up to the default limit.
    fn follow() -> Redirects {
        Redirects {
            scope: None,
            max: DEFAULT_MAX_REDIRECTIONS,
            referer: true,
        }
    }

    /// What the server received for the last request of the chain.
    fn echo(response: Result<reqwest::Response>) -> serde_json::Value {
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let text = tauri::async_runtime::block_on(response.text()).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn received<'a>(echo: &'a serde_json::Value, header: &str) -> Option<&'a str> {
        echo["headers"][header].as_str()
    }

    fn localhost_scope(port: u16) -> Option<Scope> {
        let entry = Arc::new(format!("http://localhost:{port}/*").parse().unwrap());
        Some(Scope::new(vec![entry], Vec::new()))
    }

    #[test]
    fn redirect_outside_of_scope_is_denied() {
        let port = spawn_server();

        let err = get(
            &format!("http://localhost:{port}/redirect-external"),
            localhost_scope(port),
            None,
        )
        .unwrap_err();

        match err {
            Error::UrlNotAllowed(url) => {
                assert_eq!(url.as_str(), format!("http://127.0.0.1:{port}/secret"))
            }
            e => panic!("expected the redirect to be denied by the scope, got {e:?}"),
        }
    }

    #[test]
    fn redirect_outside_of_scope_is_followed_when_not_configured() {
        let port = spawn_server();

        // `scopeRedirects` is disabled, so only the URL requested by the frontend is checked
        let response = get(
            &format!("http://localhost:{port}/redirect-external"),
            None,
            None,
        )
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn redirect_denied_by_the_scope_is_denied() {
        let port = spawn_server();
        let allow = Arc::new(format!("http://localhost:{port}/*").parse().unwrap());
        let deny = Arc::new(format!("http://localhost:{port}/target").parse().unwrap());
        let scope = Some(Scope::new(vec![allow], vec![deny]));

        let err = get(&format!("http://localhost:{port}/redirect/0"), scope, None).unwrap_err();

        assert!(matches!(err, Error::UrlNotAllowed(_)));
    }

    #[test]
    fn redirect_inside_of_scope_is_followed() {
        let port = spawn_server();

        let response = get(
            &format!("http://localhost:{port}/redirect/2"),
            localhost_scope(port),
            None,
        )
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.url().path(), "/target");
    }

    #[test]
    fn max_redirections_is_enforced() {
        let port = spawn_server();

        for scope in [localhost_scope(port), None] {
            let err = get(
                &format!("http://localhost:{port}/redirect/5"),
                scope,
                Some(2),
            )
            .unwrap_err();

            match err {
                Error::TooManyRedirects(url) => {
                    assert_eq!(url.as_str(), format!("http://localhost:{port}/redirect/2"))
                }
                e => panic!("expected too many redirects, got {e:?}"),
            }
        }
    }

    #[test]
    fn zero_max_redirections_does_not_follow() {
        let port = spawn_server();

        let response = get(
            &format!("http://localhost:{port}/redirect/0"),
            localhost_scope(port),
            Some(0),
        )
        .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[test]
    fn rule_headers_are_applied_again_on_every_hop() {
        let port = spawn_server();
        let localhost_rule = RequestHeaderRule::new(format!("http://localhost:{port}/*"))
            .header("X-Api-Key", "for-localhost")
            .header("Origin", "https://desktop.example.com");

        // a redirect within the rule's URLs keeps its headers
        let received_echo = echo(fetch(
            Method::GET,
            &format!("http://localhost:{port}/r/302/-/echo"),
            None,
            &[],
            follow(),
            std::slice::from_ref(&localhost_rule),
        ));
        assert_eq!(received(&received_echo, "x-api-key"), Some("for-localhost"));
        assert_eq!(
            received(&received_echo, "origin"),
            Some("https://desktop.example.com")
        );

        // a redirect target the rule does not match gets the plugin defaults instead
        let received_echo = echo(fetch(
            Method::GET,
            &format!("http://localhost:{port}/r/302/127.0.0.1/echo"),
            None,
            &[],
            follow(),
            std::slice::from_ref(&localhost_rule),
        ));
        assert_eq!(received(&received_echo, "x-api-key"), None);
        assert_eq!(
            received(&received_echo, "origin"),
            Some("tauri://localhost")
        );

        // and the rules matching the redirect target apply to it
        let rules = [
            localhost_rule,
            RequestHeaderRule::new(format!("http://127.0.0.1:{port}/*"))
                .header("X-Api-Key", "for-127.0.0.1"),
        ];
        let received_echo = echo(fetch(
            Method::GET,
            &format!("http://localhost:{port}/r/307/127.0.0.1/echo"),
            None,
            &[],
            follow(),
            &rules,
        ));
        assert_eq!(received(&received_echo, "x-api-key"), Some("for-127.0.0.1"));
        assert_eq!(
            received(&received_echo, "origin"),
            Some("tauri://localhost")
        );
    }

    #[test]
    fn credentials_are_not_sent_to_another_origin_for_the_rest_of_the_chain() {
        let port = spawn_server();
        let frontend = [("Authorization", "Bearer for-localhost")];

        let received_echo = echo(fetch(
            Method::GET,
            &format!("http://localhost:{port}/r/302/-/echo"),
            None,
            &frontend,
            follow(),
            &[],
        ));
        assert_eq!(
            received(&received_echo, "authorization"),
            Some("Bearer for-localhost")
        );

        // still not sent after a second, same-origin hop
        let received_echo = echo(fetch(
            Method::GET,
            &format!("http://localhost:{port}/r/302/127.0.0.1/r/302/-/echo"),
            None,
            &frontend,
            follow(),
            &[],
        ));
        assert_eq!(received_echo["path"], "/echo");
        assert_eq!(received(&received_echo, "authorization"), None);

        // the frontend's header is gone, so a rule's default for the new origin applies
        let rules = [RequestHeaderRule::new(format!("http://127.0.0.1:{port}/*"))
            .header("Authorization", "Bearer for-127.0.0.1")];
        let received_echo = echo(fetch(
            Method::GET,
            &format!("http://localhost:{port}/r/302/127.0.0.1/echo"),
            None,
            &frontend,
            follow(),
            &rules,
        ));
        assert_eq!(
            received(&received_echo, "authorization"),
            Some("Bearer for-127.0.0.1")
        );
    }

    #[test]
    fn redirects_change_the_method_and_body_like_fetch() {
        let port = spawn_server();
        let frontend = [("Content-Type", "text/plain")];
        let send = |method: Method, status: u16| {
            echo(fetch(
                method,
                &format!("http://localhost:{port}/r/{status}/-/echo"),
                Some("payload"),
                &frontend,
                follow(),
                &[],
            ))
        };

        for (method, status) in [(Method::POST, 301), (Method::POST, 302), (Method::PUT, 303)] {
            let received_echo = send(method.clone(), status);
            assert_eq!(received_echo["method"], "GET", "{method} {status}");
            assert_eq!(received_echo["body"], "", "{method} {status}");
            assert_eq!(
                received(&received_echo, "content-type"),
                None,
                "{method} {status}"
            );
        }

        for (method, status) in [(Method::PUT, 302), (Method::POST, 307), (Method::PUT, 308)] {
            let received_echo = send(method.clone(), status);
            assert_eq!(
                received_echo["method"],
                method.as_str(),
                "{method} {status}"
            );
            assert_eq!(received_echo["body"], "payload", "{method} {status}");
            assert_eq!(
                received(&received_echo, "content-type"),
                Some("text/plain"),
                "{method} {status}"
            );
        }
    }

    #[test]
    fn redirects_send_the_previous_url_as_referer() {
        let port = spawn_server();

        let received_echo = echo(get(
            &format!("http://user:pass@localhost:{port}/r/302/-/echo"),
            None,
            None,
        ));
        assert_eq!(
            received(&received_echo, "referer"),
            Some(format!("http://localhost:{port}/r/302/-/echo").as_str())
        );

        // the credentials of the URL are still sent, as basic auth
        let received_echo = echo(get(
            &format!("http://user:pass@localhost:{port}/echo"),
            None,
            None,
        ));
        assert_eq!(
            received(&received_echo, "authorization"),
            Some("Basic dXNlcjpwYXNz")
        );

        // unless disabled
        let received_echo = echo(fetch(
            Method::GET,
            &format!("http://localhost:{port}/r/302/-/echo"),
            None,
            &[],
            Redirects {
                referer: false,
                ..follow()
            },
            &[],
        ));
        assert_eq!(received(&received_echo, "referer"), None);

        let https: url::Url = "https://example.com/a".parse().unwrap();
        let http: url::Url = "http://example.com/b".parse().unwrap();
        assert!(make_referer(&http, &https).is_none());
        assert!(make_referer(&https, &http).is_some());
    }
}
