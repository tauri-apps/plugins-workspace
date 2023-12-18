// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::Deserialize;
use tauri::{command, ipc::Channel, State};

use crate::Result;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Mutex,
    },
    thread::spawn,
    time::Duration,
};

type Id = u32;

#[derive(Default)]
pub struct WatcherCollection(Mutex<HashMap<Id, (WatcherKind, Vec<PathBuf>)>>);

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

#[command]
pub async fn watch(
    watchers: State<'_, WatcherCollection>,
    id: Id,
    paths: Vec<PathBuf>,
    options: WatchOptions,
    on_event: Channel,
) -> Result<()> {
    let mode = if options.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    let watcher = if let Some(delay) = options.delay_ms {
        let (tx, rx) = channel();
        let mut debouncer = new_debouncer(Duration::from_millis(delay), tx)?;
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

    watchers.0.lock().unwrap().insert(id, (watcher, paths));

    Ok(())
}

#[command]
pub async fn unwatch(watchers: State<'_, WatcherCollection>, id: Id) -> Result<()> {
    if let Some((watcher, paths)) = watchers.0.lock().unwrap().remove(&id) {
        match watcher {
            WatcherKind::Debouncer(mut debouncer) => {
                for path in paths {
                    debouncer.watcher().unwatch(&path)?
                }
            }
            WatcherKind::Watcher(mut watcher) => {
                for path in paths {
                    watcher.unwatch(&path)?
                }
            }
        };
    }
    Ok(())
}
