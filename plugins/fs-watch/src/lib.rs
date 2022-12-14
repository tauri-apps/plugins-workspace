use notify::{
    raw_watcher, watcher, DebouncedEvent, Op, RawEvent, RecommendedWatcher, RecursiveMode,
    Watcher as _,
};
use serde::{ser::Serializer, Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri::{command, plugin::Plugin, AppHandle, Invoke, Manager, Runtime, State, Window};

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
struct WatcherCollection(Mutex<HashMap<Id, (RecommendedWatcher, Vec<PathBuf>)>>);

#[derive(Clone, Serialize)]
struct RawEventWrapper {
    path: Option<PathBuf>,
    operation: u32,
    cookie: Option<u32>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
enum DebouncedEventWrapper {
    NoticeWrite(PathBuf),
    NoticeRemove(PathBuf),
    Create(PathBuf),
    Write(PathBuf),
    Chmod(PathBuf),
    Remove(PathBuf),
    Rename(PathBuf, PathBuf),
    Rescan,
    Error {
        error: String,
        path: Option<PathBuf>,
    },
}

impl From<DebouncedEvent> for DebouncedEventWrapper {
    fn from(event: DebouncedEvent) -> Self {
        match event {
            DebouncedEvent::NoticeWrite(path) => Self::NoticeWrite(path),
            DebouncedEvent::NoticeRemove(path) => Self::NoticeRemove(path),
            DebouncedEvent::Create(path) => Self::Create(path),
            DebouncedEvent::Write(path) => Self::Write(path),
            DebouncedEvent::Chmod(path) => Self::Chmod(path),
            DebouncedEvent::Remove(path) => Self::Remove(path),
            DebouncedEvent::Rename(from, to) => Self::Rename(from, to),
            DebouncedEvent::Rescan => Self::Rescan,
            DebouncedEvent::Error(error, path) => Self::Error {
                error: error.to_string(),
                path,
            },
        }
    }
}

fn watch_raw<R: Runtime>(window: Window<R>, rx: Receiver<RawEvent>, id: Id) {
    spawn(move || {
        let event_name = format!("watcher://raw-event/{}", id);
        while let Ok(event) = rx.recv() {
            let _ = window.emit(
                &event_name,
                RawEventWrapper {
                    path: event.path,
                    operation: event.op.unwrap_or_else(|_| Op::empty()).bits(),
                    cookie: event.cookie,
                },
            );
        }
    });
}

fn watch_debounced<R: Runtime>(window: Window<R>, rx: Receiver<DebouncedEvent>, id: Id) {
    spawn(move || {
        let event_name = format!("watcher://debounced-event/{}", id);
        while let Ok(event) = rx.recv() {
            let _ = window.emit(&event_name, DebouncedEventWrapper::from(event));
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
        let mut watcher = watcher(tx, Duration::from_millis(delay))?;
        for path in &paths {
            watcher.watch(path, mode)?;
        }
        watch_debounced(window, rx, id);
        watcher
    } else {
        let (tx, rx) = channel();
        let mut watcher = raw_watcher(tx)?;
        for path in &paths {
            watcher.watch(path, mode)?;
        }
        watch_raw(window, rx, id);
        watcher
    };

    watchers.0.lock().unwrap().insert(id, (watcher, paths));

    Ok(())
}

#[command]
async fn unwatch(watchers: State<'_, WatcherCollection>, id: Id) -> Result<()> {
    if let Some((mut watcher, paths)) = watchers.0.lock().unwrap().remove(&id) {
        for path in paths {
            watcher.unwatch(path)?;
        }
    }
    Ok(())
}

/// Tauri plugin.
pub struct Watcher<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for Watcher<R> {
    fn default() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![watch, unwatch]),
        }
    }
}

impl<R: Runtime> Plugin<R> for Watcher<R> {
    fn name(&self) -> &'static str {
        "fs-watch"
    }

    fn initialize(&mut self, app: &AppHandle<R>, _config: JsonValue) -> tauri::plugin::Result<()> {
        app.manage(WatcherCollection::default());
        Ok(())
    }

    fn extend_api(&mut self, message: Invoke<R>) {
        (self.invoke_handler)(message)
    }
}
