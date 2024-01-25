// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::ScopeEntry;
use reqwest::Url;

/// Scope for filesystem access.
#[derive(Debug)]
pub struct Scope<'a> {
    allowed: Vec<&'a ScopeEntry>,
    denied: Vec<&'a ScopeEntry>,
}

impl<'a> Scope<'a> {
    /// Creates a new scope from the scope configuration.
    pub(crate) fn new(allowed: Vec<&'a ScopeEntry>, denied: Vec<&'a ScopeEntry>) -> Self {
        Self { allowed, denied }
    }

    /// Determines if the given URL is allowed on this scope.
    pub fn is_allowed(&self, url: &Url) -> bool {
        let denied = self.denied.iter().any(|entry| {
            entry.url.matches(url.as_str())
                || entry
                    .url
                    .matches(url.as_str().strip_suffix('/').unwrap_or_default())
        });
        if denied {
            false
        } else {
            self.allowed.iter().any(|entry| {
                entry.url.matches(url.as_str())
                    || entry
                        .url
                        .matches(url.as_str().strip_suffix('/').unwrap_or_default())
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::HttpAllowlistScope;

    #[test]
    fn is_allowed() {
        // plain URL
        let scope = super::Scope::new(&HttpAllowlistScope(vec!["http://localhost:8080"
            .parse()
            .unwrap()]));
        assert!(scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
        assert!(scope.is_allowed(&"http://localhost:8080/".parse().unwrap()));

        assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8080/path/to/asset.png".parse().unwrap()));
        assert!(!scope.is_allowed(&"https://localhost:8080".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8081".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://local:8080".parse().unwrap()));

        // URL with fixed path
        let scope = super::Scope::new(&HttpAllowlistScope(vec!["http://localhost:8080/file.png"
            .parse()
            .unwrap()]));

        assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));

        assert!(!scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8080/file.png/other.jpg".parse().unwrap()));

        // URL with glob pattern
        let scope = super::Scope::new(&HttpAllowlistScope(vec!["http://localhost:8080/*.png"
            .parse()
            .unwrap()]));

        assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));
        assert!(scope.is_allowed(&"http://localhost:8080/assets/file.png".parse().unwrap()));

        assert!(!scope.is_allowed(&"http://localhost:8080/file.jpeg".parse().unwrap()));

        let scope = super::Scope::new(&HttpAllowlistScope(vec!["http://*".parse().unwrap()]));

        assert!(scope.is_allowed(&"http://something.else".parse().unwrap()));
        assert!(scope.is_allowed(&"http://something.else/path/to/file".parse().unwrap()));
        assert!(!scope.is_allowed(&"https://something.else".parse().unwrap()));

        let scope = super::Scope::new(&HttpAllowlistScope(vec!["http://**".parse().unwrap()]));

        assert!(scope.is_allowed(&"http://something.else".parse().unwrap()));
        assert!(scope.is_allowed(&"http://something.else/path/to/file".parse().unwrap()));
    }
}
