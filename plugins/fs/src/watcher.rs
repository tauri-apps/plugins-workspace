// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use serde::Deserialize;
use tauri::{
    ipc::{Channel, CommandScope, GlobalScope},
    path::{BaseDirectory, SafePathBuf},
    Manager, Resource, ResourceId, Runtime, Webview,
};

use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Mutex,
    },
    thread::spawn,
    time::Duration,
};

use crate::{
    commands::{resolve_path, CommandResult},
    scope::Entry,
};

struct InnerWatcher {
    pub kind: WatcherKind,
    paths: Vec<PathBuf>,
}

pub struct WatcherResource(Mutex<InnerWatcher>);
impl WatcherResource {
    fn new(kind: WatcherKind, paths: Vec<PathBuf>) -> Self {
        Self(Mutex::new(InnerWatcher { kind, paths }))
    }

    fn with_lock<R, F: FnMut(&mut InnerWatcher) -> R>(&self, mut f: F) -> R {
        let mut watcher = self.0.lock().unwrap();
        f(&mut watcher)
    }
}

impl Resource for WatcherResource {}

enum WatcherKind {
    Debouncer(Debouncer<RecommendedWatcher, FileIdMap>),
    Watcher(RecommendedWatcher),
}

fn watch_raw(on_event: Channel<Event>, rx: Receiver<notify::Result<Event>>) {
    spawn(move || {
        while let Ok(event) = rx.recv() {
            if let Ok(event) = event {
                // TODO: Should errors be emitted too?
                let _ = on_event.send(event);
            }
        }
    });
}

fn watch_debounced(on_event: Channel<Event>, rx: Receiver<DebounceEventResult>) {
    spawn(move || {
        while let Ok(Ok(events)) = rx.recv() {
            for event in events {
                // TODO: Should errors be emitted too?
                let _ = on_event.send(event.event);
            }
        }
    });
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
    base_dir: Option<BaseDirectory>,
    recursive: bool,
    delay_ms: Option<u64>,
}

#[tauri::command]
pub async fn watch<R: Runtime>(
    webview: Webview<R>,
    paths: Vec<SafePathBuf>,
    options: WatchOptions,
    on_event: Channel<Event>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
) -> CommandResult<ResourceId> {
    let mut resolved_paths = Vec::with_capacity(paths.capacity());
    for path in paths {
        resolved_paths.push(resolve_path(
            &webview,
            &global_scope,
            &command_scope,
            path,
            options.base_dir,
        )?);
    }

    let recursive_mode = if options.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    let kind = if let Some(delay) = options.delay_ms {
        let (tx, rx) = channel();
        let mut debouncer = new_debouncer(Duration::from_millis(delay), None, tx)?;
        for path in &resolved_paths {
            debouncer.watcher().watch(path.as_ref(), recursive_mode)?;
            debouncer.cache().add_root(path, recursive_mode);
        }
        watch_debounced(on_event, rx);
        WatcherKind::Debouncer(debouncer)
    } else {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        for path in &resolved_paths {
            watcher.watch(path.as_ref(), recursive_mode)?;
        }
        watch_raw(on_event, rx);
        WatcherKind::Watcher(watcher)
    };

    let rid = webview
        .resources_table()
        .add(WatcherResource::new(kind, resolved_paths));

    Ok(rid)
}

#[tauri::command]
pub async fn unwatch<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> CommandResult<()> {
    let watcher = webview.resources_table().take::<WatcherResource>(rid)?;
    WatcherResource::with_lock(&watcher, |watcher| {
        match &mut watcher.kind {
            WatcherKind::Debouncer(ref mut debouncer) => {
                for path in &watcher.paths {
                    debouncer.watcher().unwatch(path.as_ref()).map_err(|e| {
                        format!("failed to unwatch path: {} with error: {e}", path.display())
                    })?;
                }
            }
            WatcherKind::Watcher(ref mut w) => {
                for path in &watcher.paths {
                    w.unwatch(path.as_ref()).map_err(|e| {
                        format!("failed to unwatch path: {} with error: {e}", path.display())
                    })?;
                }
            }
        }

        Ok(())
    })
}
