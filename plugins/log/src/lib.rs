// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Logging for Tauri applications.
//!
//! ## Cargo features
//!
//! - **colored**: Enables [`Builder::with_colors`] `fern`'s `colored` feature for ANSI-colored outputs.
//! - **tracing**: Emit both log and tracing for the JavaScript log commands.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use fern::{Filter, FormatCallback};
use log::{LevelFilter, Record};
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Write;
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
use tauri::{AppHandle, Emitter};
use time::{macros::format_description, OffsetDateTime};

pub use fern;
pub use log;

mod commands;

pub const WEBVIEW_TARGET: &str = "webview";

#[cfg(target_os = "ios")]
mod ios {
    swift_rs::swift!(pub fn tauri_log(
      level: u8, message: *const std::ffi::c_void
    ));
}

const DEFAULT_MAX_FILE_SIZE: u64 = 40_000;
const DEFAULT_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepOne;
const DEFAULT_TIMEZONE_STRATEGY: TimezoneStrategy = TimezoneStrategy::UseUtc;
const DEFAULT_FILE_OPEN_STRATEGY: FileOpenStrategy = FileOpenStrategy::Append;
const DEFAULT_LOG_TARGETS: [Target; 2] = [
    Target::new(TargetKind::Stdout),
    Target::new(TargetKind::LogDir { file_name: None }),
];
const LOG_DATE_FORMAT: &[time::format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day]_[hour]-[minute]-[second]");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TimeFormat(#[from] time::error::Format),
    #[error(transparent)]
    InvalidFormatDescription(#[from] time::error::InvalidFormatDescription),
    #[error("Internal logger disabled and cannot be acquired or attached")]
    LoggerNotInitialized,
}

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

#[derive(Debug, Clone)]
pub enum RotationStrategy {
    /// Will keep all the logs, renaming them to include the date.
    KeepAll,
    /// Will only keep the most recent log up to its maximal size.
    KeepOne,
    /// Will keep some of the most recent logs, renaming them to include the date.
    KeepSome(usize),
}

#[derive(Debug, Clone)]
pub enum TimezoneStrategy {
    UseUtc,
    UseLocal,
}

impl TimezoneStrategy {
    pub fn get_now(&self) -> OffsetDateTime {
        match self {
            TimezoneStrategy::UseUtc => OffsetDateTime::now_utc(),
            TimezoneStrategy::UseLocal => {
                OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
            } // Fallback to UTC since Rust cannot determine local timezone
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileOpenStrategy {
    /// Open existing file from last session and append, if any.
    Append,
    /// Create a new file on each session start, rotating the last session if any.
    Rotate,
}

/// A custom log writer that rotates the log file when it exceeds specified size.
struct RotatingFile {
    dir: PathBuf,
    file_name: String,
    path: PathBuf,
    /// Maximum file size before rotating in bytes
    max_size: u64,
    /// Current file size in bytes
    current_size: u64,
    rotation_strategy: RotationStrategy,
    timezone_strategy: TimezoneStrategy,
    file_open_strategy: FileOpenStrategy,
    inner: Option<File>,
    buffer: Vec<u8>,
}

impl RotatingFile {
    pub fn new(
        dir: impl AsRef<Path>,
        file_name: String,
        max_size: u64,
        rotation_strategy: RotationStrategy,
        timezone_strategy: TimezoneStrategy,
        file_open_strategy: FileOpenStrategy,
    ) -> Result<Self, Error> {
        let dir = dir.as_ref().to_path_buf();
        let path = dir.join(&file_name).with_extension("log");

        let mut rotator = Self {
            dir,
            file_name,
            path,
            max_size,
            current_size: 0,
            rotation_strategy,
            timezone_strategy,
            file_open_strategy,
            inner: None,
            buffer: Vec::new(),
        };

        rotator.open_file()?;
        if rotator.current_size >= rotator.max_size
            || (rotator.current_size > 0 && rotator.file_open_strategy == FileOpenStrategy::Rotate)
        {
            rotator.rotate()?;
        }
        if let RotationStrategy::KeepSome(keep_count) = rotator.rotation_strategy {
            rotator.remove_old_files(keep_count)?;
        }

        Ok(rotator)
    }

    fn open_file(&mut self) -> Result<(), Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.current_size = file.metadata()?.len();
        self.inner = Some(file);
        Ok(())
    }

    fn rotate(&mut self) -> Result<(), Error> {
        if let Some(mut file) = self.inner.take() {
            let _ = file.flush();
        }
        if self.path.exists() {
            match self.rotation_strategy {
                RotationStrategy::KeepAll => {
                    self.rename_file_to_dated()?;
                }
                RotationStrategy::KeepSome(keep_count) => {
                    // remove_old_files excludes the active file.
                    // So we need to keep (keep_count - 1) archived files to make room for the one we are about to archive.
                    self.remove_old_files(keep_count - 1)?;
                    self.rename_file_to_dated()?;
                }
                RotationStrategy::KeepOne => {
                    fs::remove_file(&self.path)?;
                }
            }
        }
        self.open_file()?;
        Ok(())
    }

    /// Remove old log files until the number of old log files is equal to the keep_count,
    /// the current active log file is not included in the keep_count.
    fn remove_old_files(&self, keep_count: usize) -> Result<(), Error> {
        let mut files = fs::read_dir(&self.dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let old_file_name = path.file_name()?.to_string_lossy().into_owned();
                if old_file_name.starts_with(&self.file_name)
                  // exclude the current active file
                  && old_file_name != format!("{}.log", self.file_name)
                {
                    let date = old_file_name
                        .strip_prefix(&self.file_name)?
                        .strip_prefix("_")?
                        .strip_suffix(".log")?;
                    Some((path, date.to_string()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        files.sort_by(|a, b| a.1.cmp(&b.1));

        if files.len() > keep_count {
            let files_to_remove = files.len() - keep_count;
            for (old_log_path, _) in files.iter().take(files_to_remove) {
                fs::remove_file(old_log_path)?;
            }
        }
        Ok(())
    }

    fn rename_file_to_dated(&self) -> Result<(), Error> {
        let to = self.dir.join(format!(
            "{}_{}.log",
            self.file_name,
            self.timezone_strategy
                .get_now()
                .format(LOG_DATE_FORMAT)
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
        fs::rename(&self.path, &to)?;
        Ok(())
    }
}

impl Write for RotatingFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        if self.inner.is_none() {
            self.open_file().map_err(std::io::Error::other)?;
        }

        if self.current_size != 0 && self.current_size + (self.buffer.len() as u64) > self.max_size
        {
            self.rotate().map_err(std::io::Error::other)?;
        }

        if let Some(file) = self.inner.as_mut() {
            file.write_all(&self.buffer)?;
            self.current_size += self.buffer.len() as u64;
            file.flush()?;
        }
        self.buffer.clear();
        Ok(())
    }
}

#[derive(Debug, Serialize, Clone)]
struct RecordPayload {
    message: String,
    level: LogLevel,
}

/// An enum representing the available targets of the logger.
pub enum TargetKind {
    /// Print logs to stdout.
    Stdout,
    /// Print logs to stderr.
    Stderr,
    /// Write logs to the given directory.
    ///
    /// The plugin will ensure the directory exists before writing logs.
    Folder {
        path: PathBuf,
        file_name: Option<String>,
    },
    /// Write logs to the OS specific logs directory.
    ///
    /// ### Platform-specific
    ///
    /// |Platform   | Value                                                                                     | Example                                                     |
    /// | --------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
    /// | Linux     | `$XDG_DATA_HOME/{bundleIdentifier}/logs` or `$HOME/.local/share/{bundleIdentifier}/logs`  | `/home/alice/.local/share/com.tauri.dev/logs`               |
    /// | macOS/iOS | `{homeDir}/Library/Logs/{bundleIdentifier}`                                               | `/Users/Alice/Library/Logs/com.tauri.dev`                   |
    /// | Windows   | `{FOLDERID_LocalAppData}/{bundleIdentifier}/logs`                                         | `C:\Users\Alice\AppData\Local\com.tauri.dev\logs`           |
    /// | Android   | `{ConfigDir}/logs`                                                                        | `/data/data/com.tauri.dev/files/logs`                       |
    LogDir { file_name: Option<String> },
    /// Forward logs to the webview (via the `log://log` event).
    ///
    /// This requires the webview to subscribe to log events, via this plugins `attachConsole` function.
    Webview,
    /// Send logs to a [`fern::Dispatch`]
    ///
    /// You can use this to construct arbitrary log targets.
    Dispatch(fern::Dispatch),
}

type Formatter = dyn Fn(FormatCallback, &Arguments, &Record) + Send + Sync + 'static;

/// A log target.
pub struct Target {
    kind: TargetKind,
    filters: Vec<Box<Filter>>,
    formatter: Option<Box<Formatter>>,
}

impl Target {
    #[inline]
    pub const fn new(kind: TargetKind) -> Self {
        Self {
            kind,
            filters: Vec::new(),
            formatter: None,
        }
    }

    #[inline]
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&log::Metadata) -> bool + Send + Sync + 'static,
    {
        self.filters.push(Box::new(filter));
        self
    }

    #[inline]
    pub fn format<F>(mut self, formatter: F) -> Self
    where
        F: Fn(FormatCallback, &Arguments, &Record) + Send + Sync + 'static,
    {
        self.formatter.replace(Box::new(formatter));
        self
    }
}

pub struct Builder {
    dispatch: fern::Dispatch,
    rotation_strategy: RotationStrategy,
    timezone_strategy: TimezoneStrategy,
    file_open_strategy: FileOpenStrategy,
    max_file_size: u128,
    targets: Vec<Target>,
    is_skip_logger: bool,
}

impl Default for Builder {
    fn default() -> Self {
        #[cfg(desktop)]
        let format = format_description!("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]");
        let dispatch = fern::Dispatch::new().format(move |out, message, record| {
            out.finish(
                #[cfg(mobile)]
                format_args!("[{}] {}", record.target(), message),
                #[cfg(desktop)]
                format_args!(
                    "{}[{}][{}] {}",
                    DEFAULT_TIMEZONE_STRATEGY.get_now().format(&format).unwrap(),
                    record.target(),
                    record.level(),
                    message
                ),
            )
        });
        Self {
            dispatch,
            rotation_strategy: DEFAULT_ROTATION_STRATEGY,
            timezone_strategy: DEFAULT_TIMEZONE_STRATEGY,
            file_open_strategy: DEFAULT_FILE_OPEN_STRATEGY,
            max_file_size: DEFAULT_MAX_FILE_SIZE as u128,
            targets: DEFAULT_LOG_TARGETS.into(),
            is_skip_logger: false,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the [`RotationStrategy`].
    ///
    /// Default is [`RotationStrategy::KeepOne`]
    pub fn rotation_strategy(mut self, rotation_strategy: RotationStrategy) -> Self {
        self.rotation_strategy = rotation_strategy;
        self
    }

    /// Sets the [`TimezoneStrategy`].
    /// Calling this method overrides the format set in [`Self::format`].
    ///
    /// Default is [`TimezoneStrategy::UseUtc`]
    pub fn timezone_strategy(mut self, timezone_strategy: TimezoneStrategy) -> Self {
        self.timezone_strategy = timezone_strategy.clone();

        let format = format_description!("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]");
        self.dispatch = self.dispatch.format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                timezone_strategy.get_now().format(&format).unwrap(),
                record.level(),
                record.target(),
                message
            ))
        });
        self
    }

    /// Sets the strategy to open the log file.
    ///
    /// The default is [`FileOpenStrategy::Append`].
    pub fn file_open_strategy(mut self, file_open_strategy: FileOpenStrategy) -> Self {
        self.file_open_strategy = file_open_strategy;
        self
    }

    /// Sets the maximum file size in bytes for log rotation.
    ///
    /// Values larger than [`u64::MAX`] will be clamped to [`u64::MAX`].
    /// In v3, this parameter will be changed to `u64`.
    ///
    /// Default is `40_000`
    pub fn max_file_size(mut self, max_file_size: u128) -> Self {
        self.max_file_size = max_file_size.min(u64::MAX as u128);
        self
    }

    /// Clears the format so that only the message is logged.
    ///
    /// e.g. `log::info!("message")` will log out `message`
    pub fn clear_format(mut self) -> Self {
        self.dispatch = self.dispatch.format(|out, message, _record| {
            out.finish(format_args!("{message}"));
        });
        self
    }

    /// Sets the formatter of this dispatch. The closure should accept a
    /// callback, a message and a log record, and write the resulting
    /// format to the writer.
    ///
    /// The log record is passed for completeness, but the `args()` method of
    /// the record should be ignored, and the [`std::fmt::Arguments`] given
    /// should be used instead. `record.args()` may be used to retrieve the
    /// _original_ log message, but in order to allow for true log
    /// chaining, formatters should use the given message instead whenever
    /// including the message in the output.
    ///
    /// To avoid all allocation of intermediate results, the formatter is
    /// "completed" by calling a callback, which then calls the rest of the
    /// logging chain with the new formatted message. The callback object keeps
    /// track of if it was called or not via a stack boolean as well, so if
    /// you don't use `out.finish` the log message will continue down
    /// the logger chain unformatted.
    ///
    /// Example usage:
    ///
    /// ```
    /// tauri_plugin_log::Builder::new()
    ///     .format(|out, message, record| {
    ///         out.finish(format_args!(
    ///             "[{} {}] {}",
    ///             record.level(),
    ///             record.target(),
    ///             message
    ///         ))
    ///     });
    /// ```
    pub fn format<F>(mut self, formatter: F) -> Self
    where
        F: Fn(FormatCallback, &Arguments, &Record) + Sync + Send + 'static,
    {
        self.dispatch = self.dispatch.format(formatter);
        self
    }

    /// Sets the overarching level filter for this logger.
    /// All messages not already filtered by something set by [`Self::level_for`] will be affected.
    ///
    /// All messages filtered will be discarded if less severe than the given level.
    ///
    /// Default level is [`log::LevelFilter::Trace`].
    pub fn level(mut self, level_filter: impl Into<LevelFilter>) -> Self {
        self.dispatch = self.dispatch.level(level_filter.into());
        self
    }

    /// Sets a per-target log level filter. Default target for log messages is
    /// `crate_name::module_name` or
    /// `crate_name` for logs in the crate root. Targets can also be set with
    /// `info!(target: "target-name", ...)`.
    ///
    /// For each log record fern will first try to match the most specific
    /// level_for, and then progressively more general ones until either a
    /// matching level is found, or the default level is used.
    ///
    /// For example, a log for the target `hyper::http::h1` will first test a
    /// level_for for `hyper::http::h1`, then for `hyper::http`, then for
    /// `hyper`, then use the default level.
    ///
    /// Examples:
    ///
    /// A program wants to include a lot of debugging output, but the library
    /// "hyper" is known to work well, so debug output from it should be
    /// excluded:
    ///
    /// ```
    /// # fn main() {
    /// tauri_plugin_log::Builder::new()
    ///     .level(log::LevelFilter::Trace)
    ///     .level_for("hyper", log::LevelFilter::Info)
    ///     # ;
    /// # }
    /// ```
    pub fn level_for(mut self, module: impl Into<Cow<'static, str>>, level: LevelFilter) -> Self {
        self.dispatch = self.dispatch.level_for(module, level);
        self
    }

    /// Adds a custom filter which can reject messages passing through this logger.
    ///
    /// [`Self::level`] and [`Self::level_for`] are preferred if applicable.
    ///
    /// Example usage:
    ///
    /// ```
    /// # fn main() {
    /// tauri_plugin_log::Builder::new()
    ///     .level(log::LevelFilter::Info)
    ///     .filter(|metadata| {
    ///         // Reject messages with the `Error` log level.
    ///         metadata.level() != log::LevelFilter::Error
    ///     })
    /// # }
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&log::Metadata) -> bool + Send + Sync + 'static,
    {
        self.dispatch = self.dispatch.filter(filter);
        self
    }

    /// Removes all targets. Useful to ignore the default targets and reconfigure them.
    pub fn clear_targets(mut self) -> Self {
        self.targets.clear();
        self
    }

    /// Adds a log target to the logger.
    ///
    /// ```rust
    /// use tauri_plugin_log::{Target, TargetKind};
    /// tauri_plugin_log::Builder::new()
    ///     .target(Target::new(TargetKind::Webview));
    /// ```
    ///
    /// The default targets are
    ///
    /// ```rust
    /// # use tauri_plugin_log::{Target, TargetKind, Builder};
    /// # Builder::new()
    /// #     .targets(
    /// [
    ///     Target::new(TargetKind::Stdout),
    ///     Target::new(TargetKind::LogDir { file_name: None }),
    /// ]
    /// #      );
    /// ```
    pub fn target(mut self, target: Target) -> Self {
        self.targets.push(target);
        self
    }

    /// Skip the creation and global registration of a logger
    ///
    /// If you wish to use your own global logger, you must call `skip_logger` so that the plugin does not attempt to set a second global logger.
    /// In this configuration, no logger will be created and the plugin's `log` command will rely on the result of `log::logger()`.
    /// You will be responsible for configuring the logger yourself and any included targets will be ignored.
    /// If ever initializing the plugin multiple times, such as if registering the plugin while testing, call this method to avoid panicking when registering multiple loggers.
    /// For interacting with `tracing`, you can leverage the `tracing-log` logger to forward logs to `tracing` or enable the `tracing` feature for this plugin to emit events directly to the tracing system.
    /// Both scenarios require calling this method.
    ///
    /// ```rust
    /// static LOGGER: SimpleLogger = SimpleLogger;
    ///
    /// log::set_logger(&SimpleLogger)?;
    /// log::set_max_level(LevelFilter::Info);
    /// tauri_plugin_log::Builder::new()
    ///     .skip_logger();
    /// ```
    pub fn skip_logger(mut self) -> Self {
        self.is_skip_logger = true;
        self
    }

    /// Replaces the targets of the logger.
    ///
    /// ```rust
    /// use tauri_plugin_log::{Target, TargetKind, WEBVIEW_TARGET};
    /// tauri_plugin_log::Builder::new()
    ///     .targets([
    ///         Target::new(TargetKind::Webview),
    ///         Target::new(TargetKind::LogDir { file_name: Some("webview".into()) }).filter(|metadata| metadata.target().starts_with(WEBVIEW_TARGET)),
    ///         Target::new(TargetKind::LogDir { file_name: Some("rust".into()) }).filter(|metadata| !metadata.target().starts_with(WEBVIEW_TARGET)),
    ///     ]);
    /// ```
    ///
    /// The default targets are
    ///
    /// ```rust
    /// # use tauri_plugin_log::{Target, TargetKind, Builder};
    /// # Builder::new()
    /// #     .targets(
    /// [
    ///     Target::new(TargetKind::Stdout),
    ///     Target::new(TargetKind::LogDir { file_name: None }),
    /// ]
    /// #      );
    /// ```
    pub fn targets(mut self, targets: impl IntoIterator<Item = Target>) -> Self {
        self.targets = Vec::from_iter(targets);
        self
    }

    #[cfg(feature = "colored")]
    pub fn with_colors(self, colors: fern::colors::ColoredLevelConfig) -> Self {
        let format = format_description!("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]");

        let timezone_strategy = self.timezone_strategy.clone();
        self.format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                timezone_strategy.get_now().format(&format).unwrap(),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
    }

    fn acquire_logger<R: Runtime>(
        app_handle: &AppHandle<R>,
        mut dispatch: fern::Dispatch,
        rotation_strategy: RotationStrategy,
        timezone_strategy: TimezoneStrategy,
        file_open_strategy: FileOpenStrategy,
        max_file_size: u64,
        targets: Vec<Target>,
    ) -> Result<(log::LevelFilter, Box<dyn log::Log>), Error> {
        let app_name = &app_handle.package_info().name;

        // setup targets
        for target in targets {
            let mut target_dispatch = fern::Dispatch::new();
            for filter in target.filters {
                target_dispatch = target_dispatch.filter(filter);
            }
            if let Some(formatter) = target.formatter {
                target_dispatch = target_dispatch.format(formatter);
            }

            let logger = match target.kind {
                #[cfg(target_os = "android")]
                TargetKind::Stdout | TargetKind::Stderr => fern::Output::call(android_logger::log),
                #[cfg(target_os = "ios")]
                TargetKind::Stdout | TargetKind::Stderr => fern::Output::call(move |record| {
                    let message = format!("{}", record.args());
                    unsafe {
                        ios::tauri_log(
                            match record.level() {
                                log::Level::Trace | log::Level::Debug => 1,
                                log::Level::Info => 2,
                                log::Level::Warn | log::Level::Error => 3,
                            },
                            // The string is allocated in rust, so we must
                            // autorelease it rust to give it to the Swift
                            // runtime.
                            objc2::rc::Retained::autorelease_ptr(
                                objc2_foundation::NSString::from_str(message.as_str()),
                            ) as _,
                        );
                    }
                }),
                #[cfg(desktop)]
                TargetKind::Stdout => std::io::stdout().into(),
                #[cfg(desktop)]
                TargetKind::Stderr => std::io::stderr().into(),
                TargetKind::Folder { path, file_name } => {
                    if !path.exists() {
                        fs::create_dir_all(&path)?;
                    }

                    let rotator = RotatingFile::new(
                        &path,
                        file_name.unwrap_or(app_name.clone()),
                        max_file_size,
                        rotation_strategy.clone(),
                        timezone_strategy.clone(),
                        file_open_strategy.clone(),
                    )?;
                    fern::Output::writer(Box::new(rotator), "\n")
                }
                TargetKind::LogDir { file_name } => {
                    let path = app_handle.path().app_log_dir()?;
                    if !path.exists() {
                        fs::create_dir_all(&path)?;
                    }

                    let rotator = RotatingFile::new(
                        &path,
                        file_name.unwrap_or(app_name.clone()),
                        max_file_size,
                        rotation_strategy.clone(),
                        timezone_strategy.clone(),
                        file_open_strategy.clone(),
                    )?;
                    fern::Output::writer(Box::new(rotator), "\n")
                }
                TargetKind::Webview => {
                    let app_handle = app_handle.clone();

                    fern::Output::call(move |record| {
                        let payload = RecordPayload {
                            message: record.args().to_string(),
                            level: record.level().into(),
                        };
                        let app_handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = app_handle.emit("log://log", payload);
                        });
                    })
                }
                TargetKind::Dispatch(dispatch) => dispatch.into(),
            };
            target_dispatch = target_dispatch.chain(logger);

            dispatch = dispatch.chain(target_dispatch);
        }

        Ok(dispatch.into_log())
    }

    fn plugin_builder<R: Runtime>() -> plugin::Builder<R> {
        plugin::Builder::new("log").invoke_handler(tauri::generate_handler![commands::log])
    }

    #[allow(clippy::type_complexity)]
    pub fn split<R: Runtime>(
        self,
        app_handle: &AppHandle<R>,
    ) -> Result<(TauriPlugin<R>, log::LevelFilter, Box<dyn log::Log>), Error> {
        if self.is_skip_logger {
            return Err(Error::LoggerNotInitialized);
        }
        let plugin = Self::plugin_builder();
        let (max_level, log) = Self::acquire_logger(
            app_handle,
            self.dispatch,
            self.rotation_strategy,
            self.timezone_strategy,
            self.file_open_strategy,
            self.max_file_size as u64,
            self.targets,
        )?;

        Ok((plugin.build(), max_level, log))
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        Self::plugin_builder()
            .setup(move |app_handle, _api| {
                if !self.is_skip_logger {
                    let (max_level, log) = Self::acquire_logger(
                        app_handle,
                        self.dispatch,
                        self.rotation_strategy,
                        self.timezone_strategy,
                        self.file_open_strategy,
                        self.max_file_size as u64,
                        self.targets,
                    )?;
                    attach_logger(max_level, log)?;
                }
                Ok(())
            })
            .build()
    }
}

/// Attaches the given logger
pub fn attach_logger(
    max_level: log::LevelFilter,
    log: Box<dyn log::Log>,
) -> Result<(), log::SetLoggerError> {
    log::set_boxed_logger(log)?;
    log::set_max_level(max_level);
    Ok(())
}
