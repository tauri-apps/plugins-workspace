// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Entry {
    pub path: Option<PathBuf>,
}

pub type EventId = u32;

/// Scope change event.
#[derive(Debug, Clone)]
pub enum Event {
    /// A path has been allowed.
    PathAllowed(PathBuf),
    /// A path has been forbidden.
    PathForbidden(PathBuf),
}

#[derive(Default)]
pub struct Scope {
    // TODO: Remove Option in v2, just used to keep Default
    pub(crate) scope: Option<tauri::fs::Scope>,
    pub(crate) require_literal_leading_dot: Option<bool>,
}

impl Scope {
    /// Extend the allowed patterns with the given directory.
    ///
    /// After this function has been called, the frontend will be able to use the Tauri API to read
    /// the directory and all of its files. If `recursive` is `true`, subdirectories will be accessible too.
    // TODO: Return Result
    pub fn allow_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) {
        if let Some(scope) = &self.scope {
            let _ = scope.allow_directory(path, recursive);
        }
    }

    /// Extend the allowed patterns with the given file path.
    ///
    /// After this function has been called, the frontend will be able to use the Tauri API to read the contents of this file.
    // TODO: Return Result
    pub fn allow_file<P: AsRef<Path>>(&self, path: P) {
        if let Some(scope) = &self.scope {
            let _ = scope.allow_file(path);
        }
    }

    /// Set the given directory path to be forbidden by this scope.
    ///
    /// **Note:** this takes precedence over allowed paths, so its access gets denied **always**.
    // TODO: Return Result
    pub fn forbid_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) {
        if let Some(scope) = &self.scope {
            let _ = scope.forbid_directory(path, recursive);
        }
    }

    /// Set the given file path to be forbidden by this scope.
    ///
    /// **Note:** this takes precedence over allowed paths, so its access gets denied **always**.
    // TODO: Return Result
    pub fn forbid_file<P: AsRef<Path>>(&self, path: P) {
        if let Some(scope) = &self.scope {
            let _ = scope.forbid_file(path);
        }
    }

    /// List of allowed paths.
    #[deprecated(since = "2.1.0", note = "use `allowed_patterns` instead")]
    pub fn allowed(&self) -> Vec<PathBuf> {
        self.scope
            .as_ref()
            .map(|s| s.allowed_patterns().clone())
            .unwrap_or_default()
            .iter()
            .map(|p| PathBuf::from(p.as_str()))
            .collect()
    }

    /// List of allowed patterns. Note that this does not include paths defined in capabilites.
    pub fn allowed_patterns(&self) -> HashSet<tauri::fs::Pattern> {
        self.scope
            .as_ref()
            .map(|s| s.allowed_patterns().clone())
            .unwrap_or_default()
    }

    /// List of forbidden paths.
    #[deprecated(since = "2.1.0", note = "use `forbidden_patterns` instead")]
    pub fn forbidden(&self) -> Vec<PathBuf> {
        self.scope
            .as_ref()
            .map(|s| s.forbidden_patterns().clone())
            .unwrap_or_default()
            .iter()
            .map(|p| PathBuf::from(p.as_str()))
            .collect()
    }

    /// List of forbidden patterns. Note that this does not include paths defined in capabilites.
    pub fn forbidden_patterns(&self) -> HashSet<tauri::fs::Pattern> {
        self.scope
            .as_ref()
            .map(|s| s.forbidden_patterns())
            .unwrap_or_default()
    }

    /// Listen to an event on this scope.
    /// Silently fails and returns `0` until v3 if `Scope` was constructed manually instead of getting it via `app.fs_scope()`.
    pub fn listen<F: Fn(&Event) + Send + 'static>(&self, f: F) -> EventId {
        if let Some(scope) = &self.scope {
            scope.listen(move |e| match e {
                tauri::fs::Event::PathAllowed(p) => f(&Event::PathAllowed(p.to_owned())),
                tauri::fs::Event::PathForbidden(p) => f(&Event::PathForbidden(p.to_owned())),
            })
        } else {
            0
        }
    }
}
