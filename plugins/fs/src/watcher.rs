// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, RecommendedCache};
use serde::Deserialize;
use tauri::{
    ipc::{Channel, CommandScope, GlobalScope},
    path::BaseDirectory,
    Manager, Resource, ResourceId, Runtime, Webview,
};

use std::time::Duration;

use crate::{
    commands::{resolve_path, CommandResult},
    scope::Entry,
    SafeFilePath,
};

#[allow(unused)]
enum WatcherKind {
    Debouncer(Debouncer<RecommendedWatcher, RecommendedCache>),
    Watcher(RecommendedWatcher),
}

impl Resource for WatcherKind {}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
    base_dir: Option<BaseDirectory>,
    #[serde(default)]
    recursive: bool,
    delay_ms: Option<u64>,
}

#[tauri::command]
pub fn watch<R: Runtime>(
    webview: Webview<R>,
    paths: Vec<SafeFilePath>,
    options: WatchOptions,
    on_event: Channel<notify::Event>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
) -> CommandResult<ResourceId> {
    let resolved_paths = paths
        .into_iter()
        .map(|path| {
            resolve_path(
                "watch",
                &webview,
                &global_scope,
                &command_scope,
                path,
                options.base_dir,
            )
        })
        .collect::<CommandResult<Vec<_>>>()?;

    let recursive_mode = if options.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    let watcher_kind = if let Some(delay) = options.delay_ms {
        let mut debouncer = new_debouncer(
            Duration::from_millis(delay),
            None,
            move |events: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| {
                if let Ok(events) = events {
                    for event in events {
                        // TODO: Should errors be emitted too?
                        let _ = on_event.send(event.event);
                    }
                }
            },
        )?;
        for path in &resolved_paths {
            debouncer.watch(path, recursive_mode)?;
        }
        WatcherKind::Debouncer(debouncer)
    } else {
        let mut watcher = RecommendedWatcher::new(
            move |event| {
                if let Ok(event) = event {
                    // TODO: Should errors be emitted too?
                    let _ = on_event.send(event);
                }
            },
            Config::default(),
        )?;
        for path in &resolved_paths {
            watcher.watch(path, recursive_mode)?;
        }
        WatcherKind::Watcher(watcher)
    };

    let rid = webview.resources_table().add(watcher_kind);

    Ok(rid)
}
