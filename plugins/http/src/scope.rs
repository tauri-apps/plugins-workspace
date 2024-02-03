// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Deserializer};
use url::Url;

#[allow(rustdoc::bare_urls)]
#[derive(Debug)]
pub struct Entry {
    pub url: glob::Pattern,
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EntryRaw {
            url: String,
        }

        EntryRaw::deserialize(deserializer).and_then(|raw| {
            Ok(Entry {
                url: glob::Pattern::new(&raw.url).map_err(|e| {
                    serde::de::Error::custom(format!(
                        "URL `{}` is not a valid glob pattern: {e}",
                        raw.url
                    ))
                })?,
            })
        })
    }
}

/// Scope for filesystem access.
#[derive(Debug)]
pub struct Scope<'a> {
    allowed: Vec<&'a Entry>,
    denied: Vec<&'a Entry>,
}

impl<'a> Scope<'a> {
    /// Creates a new scope from the scope configuration.
    pub(crate) fn new(allowed: Vec<&'a Entry>, denied: Vec<&'a Entry>) -> Self {
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
    use std::str::FromStr;

    use super::Entry;

    impl FromStr for Entry {
        type Err = glob::PatternError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let pattern = s.parse()?;
            Ok(Self { url: pattern })
        }
    }

    #[test]
    fn is_allowed() {
        // plain URL
        let entry = "http://localhost:8080".parse().unwrap();
        let scope = super::Scope::new(vec![&entry], Vec::new());
        assert!(scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
        assert!(scope.is_allowed(&"http://localhost:8080/".parse().unwrap()));

        assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8080/path/to/asset.png".parse().unwrap()));
        assert!(!scope.is_allowed(&"https://localhost:8080".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8081".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://local:8080".parse().unwrap()));

        // deny takes precedence
        let allow = "http://localhost:8080/file.png".parse().unwrap();
        let deny = "http://localhost:8080/*".parse().unwrap();
        let scope = super::Scope::new(vec![&allow], vec![&deny]);
        assert!(!scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));

        // URL with fixed path
        let entry = "http://localhost:8080/file.png".parse().unwrap();
        let scope = super::Scope::new(vec![&entry], Vec::new());

        assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));

        assert!(!scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
        assert!(!scope.is_allowed(&"http://localhost:8080/file.png/other.jpg".parse().unwrap()));

        // URL with glob pattern
        let entry = "http://localhost:8080/*.png".parse().unwrap();
        let scope = super::Scope::new(vec![&entry], Vec::new());

        assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));
        assert!(scope.is_allowed(&"http://localhost:8080/assets/file.png".parse().unwrap()));

        assert!(!scope.is_allowed(&"http://localhost:8080/file.jpeg".parse().unwrap()));

        let entry = "http://*".parse().unwrap();
        let scope = super::Scope::new(vec![&entry], Vec::new());

        assert!(scope.is_allowed(&"http://something.else".parse().unwrap()));
        assert!(scope.is_allowed(&"http://something.else/path/to/file".parse().unwrap()));
        assert!(!scope.is_allowed(&"https://something.else".parse().unwrap()));

        let entry = "http://**".parse().unwrap();
        let scope = super::Scope::new(vec![&entry], Vec::new());

        assert!(scope.is_allowed(&"http://something.else".parse().unwrap()));
        assert!(scope.is_allowed(&"http://something.else/path/to/file".parse().unwrap()));
    }
}
