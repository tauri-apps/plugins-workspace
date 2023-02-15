// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use fern::FormatCallback;
use log::{logger, RecordBuilder};
use log::{LevelFilter, Record};
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::borrow::Cow;
use std::collections::HashMap;
use std::{
    fmt::Arguments,
    fs::{self, File},
    iter::FromIterator,
    path::{Path, PathBuf},
};
use tauri::{
    plugin::{self, TauriPlugin},
    Manager, Runtime,
};

pub use fern;

const DEFAULT_MAX_FILE_SIZE: u128 = 40000;
const DEFAULT_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepOne;
const DEFAULT_LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::LogDir];

/// An enum representing the available verbosity levels of the logger.
///
/// It is very similar to the [`log::Level`], but serializes to unsigned ints instead of strings.
#[derive(Debug, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u16)]
pub enum LogLevel {
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace = 1,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error,
}

impl From<LogLevel> for log::Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Trace => log::Level::Trace,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        }
    }
}

impl From<log::Level> for LogLevel {
    fn from(log_level: log::Level) -> Self {
        match log_level {
            log::Level::Trace => LogLevel::Trace,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Info => LogLevel::Info,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Error => LogLevel::Error,
        }
    }
}

pub enum RotationStrategy {
    KeepAll,
    KeepOne,
}

#[derive(Debug, Serialize, Clone)]
struct RecordPayload {
    message: String,
    level: LogLevel,
}

/// An enum representing the available targets of the logger.
pub enum LogTarget {
    /// Print logs to stdout.
    Stdout,
    /// Print logs to stderr.
    Stderr,
    /// Write logs to the given directory.
    ///
    /// The plugin will ensure the directory exists before writing logs.
    Folder(PathBuf),
    /// Write logs to the OS specififc logs directory.
    ///
    /// ### Platform-specific
    ///
    /// |Platform | Value                                         | Example                                        |
    /// | ------- | --------------------------------------------- | ---------------------------------------------- |
    /// | Linux   | `{configDir}/{bundleIdentifier}`              | `/home/alice/.config/com.tauri.dev`            |
    /// | macOS   | `{homeDir}/Library/Logs/{bundleIdentifier}`   | `/Users/Alice/Library/Logs/com.tauri.dev`      |
    /// | Windows | `{configDir}/{bundleIdentifier}`              | `C:\Users\Alice\AppData\Roaming\com.tauri.dev` |
    LogDir,
    /// Forward logs to the webview (via the `log://log` event).
    ///
    /// This requires the webview to subscribe to log events, via this plugins `attachConsole` function.
    Webview,
}

#[tauri::command]
fn log(
    level: LogLevel,
    message: String,
    location: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
    key_values: Option<HashMap<String, String>>,
) {
    let location = location.unwrap_or("webview");
    let mut builder = RecordBuilder::new();
    builder
        .target(location)
        .level(level.into())
        .file(file)
        .line(line);

    let key_values = key_values.unwrap_or_default();
    let mut kv = HashMap::new();
    for (k, v) in key_values.iter() {
        kv.insert(k.as_str(), v.as_str());
    }
    builder.key_values(&kv);

    logger().log(&builder.args(format_args!("{message}")).build());
}

pub struct Builder {
    dispatch: fern::Dispatch,
    rotation_strategy: RotationStrategy,
    max_file_size: u128,
    targets: Vec<LogTarget>,
}

impl Default for Builder {
    fn default() -> Self {
        #[cfg(desktop)]
        let format =
            time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
                .unwrap();
        let dispatch = fern::Dispatch::new().format(move |out, message, record| {
            out.finish(
                #[cfg(mobile)]
                format_args!("[{}] {}", record.target(), message),
                #[cfg(desktop)]
                format_args!(
                    "{}[{}][{}] {}",
                    time::OffsetDateTime::now_utc().format(&format).unwrap(),
                    record.target(),
                    record.level(),
                    message
                ),
            )
        });
        Self {
            dispatch,
            rotation_strategy: DEFAULT_ROTATION_STRATEGY,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            targets: DEFAULT_LOG_TARGETS.into(),
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn rotation_strategy(mut self, rotation_strategy: RotationStrategy) -> Self {
        self.rotation_strategy = rotation_strategy;
        self
    }

    pub fn max_file_size(mut self, max_file_size: u128) -> Self {
        self.max_file_size = max_file_size;
        self
    }

    pub fn format<F>(mut self, formatter: F) -> Self
    where
        F: Fn(FormatCallback, &Arguments, &Record) + Sync + Send + 'static,
    {
        self.dispatch = self.dispatch.format(formatter);
        self
    }

    pub fn level(mut self, level_filter: impl Into<LevelFilter>) -> Self {
        self.dispatch = self.dispatch.level(level_filter.into());
        self
    }

    pub fn level_for(mut self, module: impl Into<Cow<'static, str>>, level: LevelFilter) -> Self {
        self.dispatch = self.dispatch.level_for(module, level);
        self
    }

    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&log::Metadata) -> bool + Send + Sync + 'static,
    {
        self.dispatch = self.dispatch.filter(filter);
        self
    }

    pub fn target(mut self, target: LogTarget) -> Self {
        self.targets.push(target);
        self
    }

    pub fn targets(mut self, targets: impl IntoIterator<Item = LogTarget>) -> Self {
        self.targets = Vec::from_iter(targets);
        self
    }

    #[cfg(feature = "colored")]
    pub fn with_colors(self, colors: fern::colors::ColoredLevelConfig) -> Self {
        let format =
            time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
                .unwrap();
        self.format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                time::OffsetDateTime::now_utc().format(&format).unwrap(),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
    }

    pub fn build<R: Runtime>(mut self) -> TauriPlugin<R> {
        plugin::Builder::new("log")
            .invoke_handler(tauri::generate_handler![log])
            .setup(move |app_handle| {
                let app_name = &app_handle.package_info().name;

                // setup targets
                for target in &self.targets {
                    let logger = match target {
                        #[cfg(target_os = "android")]
                        LogTarget::Stdout | LogTarget::Stderr => {
                            fern::Output::call(android_logger::log)
                        }
                        #[cfg(target_os = "ios")]
                        LogTarget::Stdout | LogTarget::Stderr => {
                            use std::sync::Mutex;
                            let loggers: Mutex<HashMap<String, oslog::OsLog>> = Default::default();
                            let mut subsystem = String::new();
                            let identifier = &app_handle.config().tauri.bundle.identifier;
                            let s = identifier.split('.');
                            let last = s.clone().count() - 1;
                            for (i, w) in s.enumerate() {
                                if i != last {
                                    subsystem.push_str(w);
                                    subsystem.push('.');
                                }
                            }
                            subsystem.push_str(&app_handle.package_info().crate_name);

                            fern::Output::call(move |record| {
                                let mut loggers = loggers.lock().unwrap();
                                let pair =
                                    loggers.entry(record.target().into()).or_insert_with(|| {
                                        oslog::OsLog::new(&subsystem, record.target())
                                    });

                                let message = format!("{}", record.args());
                                (*pair).with_level(record.level().into(), &message);
                            })
                        }
                        #[cfg(desktop)]
                        LogTarget::Stdout => std::io::stdout().into(),
                        #[cfg(desktop)]
                        LogTarget::Stderr => std::io::stderr().into(),
                        LogTarget::Folder(path) => {
                            if !path.exists() {
                                fs::create_dir_all(path).unwrap();
                            }

                            fern::log_file(get_log_file_path(
                                &path,
                                app_name,
                                &self.rotation_strategy,
                                self.max_file_size,
                            )?)?
                            .into()
                        }
                        #[cfg(mobile)]
                        LogTarget::LogDir => continue,
                        #[cfg(desktop)]
                        LogTarget::LogDir => {
                            let path = app_handle.path_resolver().app_log_dir().unwrap();
                            if !path.exists() {
                                fs::create_dir_all(&path).unwrap();
                            }

                            fern::log_file(get_log_file_path(
                                &path,
                                app_name,
                                &self.rotation_strategy,
                                self.max_file_size,
                            )?)?
                            .into()
                        }
                        LogTarget::Webview => {
                            let app_handle = app_handle.clone();

                            fern::Output::call(move |record| {
                                let payload = RecordPayload {
                                    message: record.args().to_string(),
                                    level: record.level().into(),
                                };
                                let app_handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    app_handle.emit_all("log://log", payload).unwrap();
                                });
                            })
                        }
                    };
                    self.dispatch = self.dispatch.chain(logger);
                }

                self.dispatch.apply()?;

                Ok(())
            })
            .build()
    }
}

fn get_log_file_path(
    dir: &impl AsRef<Path>,
    app_name: &str,
    rotation_strategy: &RotationStrategy,
    max_file_size: u128,
) -> plugin::Result<PathBuf> {
    let path = dir.as_ref().join(format!("{app_name}.log"));

    if path.exists() {
        let log_size = File::open(&path)?.metadata()?.len() as u128;
        if log_size > max_file_size {
            match rotation_strategy {
                RotationStrategy::KeepAll => {
                    let to = dir.as_ref().join(format!(
                        "{}_{}.log",
                        app_name,
                        time::OffsetDateTime::now_utc()
                            .format(
                                &time::format_description::parse(
                                    "[year]-[month]-[day]_[hour]-[minute]-[second]"
                                )
                                .unwrap()
                            )
                            .unwrap(),
                    ));
                    if to.is_file() {
                        // designated rotated log file name already exists
                        // highly unlikely but defensively handle anyway by adding .bak to filename
                        let mut to_bak = to.clone();
                        to_bak.set_file_name(format!(
                            "{}.bak",
                            to_bak.file_name().unwrap().to_string_lossy()
                        ));
                        fs::rename(&to, to_bak)?;
                    }
                    fs::rename(&path, to)?;
                }
                RotationStrategy::KeepOne => {
                    fs::remove_file(&path)?;
                }
            }
        }
    }

    Ok(path)
}
