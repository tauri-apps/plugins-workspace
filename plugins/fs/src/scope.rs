// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};

use serde::Deserialize;

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum EntryRaw {
    Value(PathBuf),
    Object { path: PathBuf },
}

#[derive(Debug)]
pub struct Entry {
    pub path: Option<PathBuf>,
}

pub type EventId = u32;
type EventListener = Box<dyn Fn(&Event) + Send>;

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
    pub(crate) allowed: Mutex<Vec<PathBuf>>,
    pub(crate) denied: Mutex<Vec<PathBuf>>,
    event_listeners: Mutex<HashMap<EventId, EventListener>>,
    next_event_id: AtomicU32,
    pub(crate) require_literal_leading_dot: Option<bool>,
}

impl Scope {
    /// Extend the allowed patterns with the given directory.
    ///
    /// After this function has been called, the frontend will be able to use the Tauri API to read
    /// the directory and all of its files. If `recursive` is `true`, subdirectories will be accessible too.
    pub fn allow_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) {
        let path = path.as_ref();

        {
            let mut allowed = self.allowed.lock().unwrap();
            allowed.push(path.to_path_buf());
            allowed.push(path.join(if recursive { "**" } else { "*" }));
        }

        self.emit(Event::PathAllowed(path.to_path_buf()));
    }

    /// Extend the allowed patterns with the given file path.
    ///
    /// After this function has been called, the frontend will be able to use the Tauri API to read the contents of this file.
    pub fn allow_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();

        self.allowed.lock().unwrap().push(path.to_path_buf());

        self.emit(Event::PathAllowed(path.to_path_buf()));
    }

    /// Set the given directory path to be forbidden by this scope.
    ///
    /// **Note:** this takes precedence over allowed paths, so its access gets denied **always**.
    pub fn forbid_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) {
        let path = path.as_ref();

        {
            let mut denied = self.denied.lock().unwrap();
            denied.push(path.to_path_buf());
            denied.push(path.join(if recursive { "**" } else { "*" }));
        }

        self.emit(Event::PathForbidden(path.to_path_buf()));
    }

    /// Set the given file path to be forbidden by this scope.
    ///
    /// **Note:** this takes precedence over allowed paths, so its access gets denied **always**.
    pub fn forbid_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();

        self.denied.lock().unwrap().push(path.to_path_buf());

        self.emit(Event::PathForbidden(path.to_path_buf()));
    }

    /// List of allowed paths.
    pub fn allowed(&self) -> Vec<PathBuf> {
        self.allowed.lock().unwrap().clone()
    }

    /// List of forbidden paths.
    pub fn forbidden(&self) -> Vec<PathBuf> {
        self.denied.lock().unwrap().clone()
    }

    fn next_event_id(&self) -> u32 {
        self.next_event_id.fetch_add(1, Ordering::Relaxed)
    }

    fn emit(&self, event: Event) {
        let listeners = self.event_listeners.lock().unwrap();
        let handlers = listeners.values();
        for listener in handlers {
            listener(&event);
        }
    }

    /// Listen to an event on this scope.
    pub fn listen<F: Fn(&Event) + Send + 'static>(&self, f: F) -> EventId {
        let id = self.next_event_id();
        self.event_listeners.lock().unwrap().insert(id, Box::new(f));
        id
    }
}
