// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::Deserialize;
use tauri::{ipc::Channel, AppHandle, Manager, Resource, ResourceId, Runtime};

use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Mutex,
    },
    thread::spawn,
    time::Duration,
};

use crate::commands::CommandResult;

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
    Debouncer(Debouncer<RecommendedWatcher>),
    Watcher(RecommendedWatcher),
}

fn watch_raw(on_event: Channel, rx: Receiver<notify::Result<Event>>) {
    spawn(move || {
        while let Ok(event) = rx.recv() {
            if let Ok(event) = event {
                // TODO: Should errors be emitted too?
                let _ = on_event.send(&event);
            }
        }
    });
}

fn watch_debounced(on_event: Channel, rx: Receiver<DebounceEventResult>) {
    spawn(move || {
        while let Ok(event) = rx.recv() {
            if let Ok(event) = event {
                // TODO: Should errors be emitted too?
                let _ = on_event.send(&event);
            }
        }
    });
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
    delay_ms: Option<u64>,
    recursive: bool,
}

#[tauri::command]
pub async fn watch<R: Runtime>(
    app: AppHandle<R>,
    paths: Vec<PathBuf>,
    options: WatchOptions,
    on_event: Channel,
) -> CommandResult<ResourceId> {
    let mode = if options.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    let kind = if let Some(delay) = options.delay_ms {
        let (tx, rx) = channel();
        let mut debouncer = new_debouncer(Duration::from_millis(delay), None, tx)?;
        let watcher = debouncer.watcher();
        for path in &paths {
            watcher.watch(path, mode)?;
        }
        watch_debounced(on_event, rx);
        WatcherKind::Debouncer(debouncer)
    } else {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        for path in &paths {
            watcher.watch(path, mode)?;
        }
        watch_raw(on_event, rx);
        WatcherKind::Watcher(watcher)
    };

    let rid = app.resources_table().add(WatcherResource::new(kind, paths));

    Ok(rid)
}

#[tauri::command]
pub async fn unwatch<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> CommandResult<()> {
    let watcher = app.resources_table().take::<WatcherResource>(rid)?;
    WatcherResource::with_lock(&watcher, |watcher| {
        match &mut watcher.kind {
            WatcherKind::Debouncer(ref mut debouncer) => {
                for path in &watcher.paths {
                    debouncer.watcher().unwatch(path).map_err(|e| {
                        format!("failed to unwatch path: {} with error: {e}", path.display())
                    })?
                }
            }
            WatcherKind::Watcher(ref mut w) => {
                for path in &watcher.paths {
                    w.unwatch(path).map_err(|e| {
                        format!("failed to unwatch path: {} with error: {e}", path.display())
                    })?
                }
            }
        }

        Ok(())
    })
}
