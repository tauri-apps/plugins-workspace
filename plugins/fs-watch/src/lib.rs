use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::{ser::Serializer, Deserialize, Serialize};
use tauri::{
    command,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State, Window,
};

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

type Result<T> = std::result::Result<T, Error>;
type Id = u32;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Watch(#[from] notify::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Default)]
struct WatcherCollection(Mutex<HashMap<Id, (WatcherKind, Vec<PathBuf>)>>);

enum WatcherKind {
    Debouncer(Debouncer<RecommendedWatcher>),
    Watcher(RecommendedWatcher),
}

fn watch_raw<R: Runtime>(window: Window<R>, rx: Receiver<notify::Result<Event>>, id: Id) {
    spawn(move || {
        let event_name = format!("watcher://raw-event/{id}");
        while let Ok(event) = rx.recv() {
            if let Ok(event) = event {
                // TODO: Should errors be emitted too?
                let _ = window.emit(&event_name, event);
            }
        }
    });
}

fn watch_debounced<R: Runtime>(window: Window<R>, rx: Receiver<DebounceEventResult>, id: Id) {
    spawn(move || {
        let event_name = format!("watcher://debounced-event/{id}");
        while let Ok(event) = rx.recv() {
            if let Ok(event) = event {
                // TODO: Should errors be emitted too?
                let _ = window.emit(&event_name, event);
            }
        }
    });
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WatchOptions {
    delay_ms: Option<u64>,
    recursive: bool,
}

#[command]
async fn watch<R: Runtime>(
    window: Window<R>,
    watchers: State<'_, WatcherCollection>,
    id: Id,
    paths: Vec<PathBuf>,
    options: WatchOptions,
) -> Result<()> {
    let mode = if options.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    let watcher = if let Some(delay) = options.delay_ms {
        let (tx, rx) = channel();
        let mut debouncer = new_debouncer(Duration::from_millis(delay), None, tx)?;
        let watcher = debouncer.watcher();
        for path in &paths {
            watcher.watch(path, mode)?;
        }
        watch_debounced(window, rx, id);
        WatcherKind::Debouncer(debouncer)
    } else {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        for path in &paths {
            watcher.watch(path, mode)?;
        }
        watch_raw(window, rx, id);
        WatcherKind::Watcher(watcher)
    };

    watchers.0.lock().unwrap().insert(id, (watcher, paths));

    Ok(())
}

#[command]
async fn unwatch(watchers: State<'_, WatcherCollection>, id: Id) -> Result<()> {
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

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("fs-watch")
        .invoke_handler(tauri::generate_handler![watch, unwatch])
        .setup(|app, _api| {
            app.manage(WatcherCollection::default());
            Ok(())
        })
        .build()
}
