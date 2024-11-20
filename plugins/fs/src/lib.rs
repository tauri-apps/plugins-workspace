// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access the file system.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use std::io::Read;

use serde::Deserialize;
use tauri::{
    ipc::ScopeObject,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    utils::{acl::Value, config::FsScope},
    AppHandle, DragDropEvent, Manager, RunEvent, Runtime, WindowEvent,
};

mod commands;
mod config;
#[cfg(not(target_os = "android"))]
mod desktop;
mod error;
mod file_path;
#[cfg(target_os = "android")]
mod mobile;
#[cfg(target_os = "android")]
mod models;
mod scope;
#[cfg(feature = "watch")]
mod watcher;

#[cfg(not(target_os = "android"))]
pub use desktop::Fs;
#[cfg(target_os = "android")]
pub use mobile::Fs;

pub use error::Error;

pub use file_path::FilePath;
pub use file_path::SafeFilePath;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOptions {
    #[serde(default = "default_true")]
    read: bool,
    #[serde(default)]
    write: bool,
    #[serde(default)]
    append: bool,
    #[serde(default)]
    truncate: bool,
    #[serde(default)]
    create: bool,
    #[serde(default)]
    create_new: bool,
    #[serde(default)]
    #[allow(unused)]
    mode: Option<u32>,
    #[serde(default)]
    #[allow(unused)]
    custom_flags: Option<i32>,
}

fn default_true() -> bool {
    true
}

impl From<OpenOptions> for std::fs::OpenOptions {
    fn from(open_options: OpenOptions) -> Self {
        let mut opts = std::fs::OpenOptions::new();

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if let Some(mode) = open_options.mode {
                opts.mode(mode);
            }
            if let Some(flags) = open_options.custom_flags {
                opts.custom_flags(flags);
            }
        }

        opts.read(open_options.read)
            .write(open_options.write)
            .create(open_options.create)
            .append(open_options.append)
            .truncate(open_options.truncate)
            .create_new(open_options.create_new);

        opts
    }
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_fs::OpenOptions;
    ///
    /// let mut options = OpenOptions::new();
    /// let file = options.read(true).open("foo.txt");
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `read`-able if opened.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("foo.txt");
    /// ```
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `write`-able if opened.
    ///
    /// If the file already exists, any write calls on it will overwrite its
    /// contents, without truncating it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).open("foo.txt");
    /// ```
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as
    /// setting only `.append(true)`.
    ///
    /// Append mode guarantees that writes will be positioned at the current end of file,
    /// even when there are other processes or threads appending to the same file. This is
    /// unlike <code>[seek]\([SeekFrom]::[End]\(0))</code> followed by `write()`, which
    /// has a race between seeking and writing during which another writer can write, with
    /// our `write()` overwriting their data.
    ///
    /// Keep in mind that this does not necessarily guarantee that data appended by
    /// different processes or threads does not interleave. The amount of data accepted a
    /// single `write()` call depends on the operating system and file system. A
    /// successful `write()` is allowed to write only part of the given data, so even if
    /// you're careful to provide the whole message in a single call to `write()`, there
    /// is no guarantee that it will be written out in full. If you rely on the filesystem
    /// accepting the message in a single write, make sure that all data that belongs
    /// together is written in one operation. This can be done by concatenating strings
    /// before passing them to [`write()`].
    ///
    /// If a file is opened with both read and append access, beware that after
    /// opening, and after every write, the position for reading may be set at the
    /// end of the file. So, before writing, save the current position (using
    /// <code>[Seek]::[stream_position]</code>), and restore it before the next read.
    ///
    /// ## Note
    ///
    /// This function doesn't create the file if it doesn't exist. Use the
    /// [`OpenOptions::create`] method to do so.
    ///
    /// [`write()`]: Write::write "io::Write::write"
    /// [`flush()`]: Write::flush "io::Write::flush"
    /// [stream_position]: Seek::stream_position "io::Seek::stream_position"
    /// [seek]: Seek::seek "io::Seek::seek"
    /// [Current]: SeekFrom::Current "io::SeekFrom::Current"
    /// [End]: SeekFrom::End "io::SeekFrom::End"
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().append(true).open("foo.txt");
    /// ```
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set it will truncate
    /// the file to 0 length if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).truncate(true).open("foo.txt");
    /// ```
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    ///
    /// In order for the file to be created, [`OpenOptions::write`] or
    /// [`OpenOptions::append`] access must be used.
    ///
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).create(true).open("foo.txt");
    /// ```
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    ///
    /// No file is allowed to exist at the target location, also no (dangling) symlink. In this
    /// way, if the call succeeds, the file returned is guaranteed to be new.
    /// If a file exists at the target location, creating a new file will fail with [`AlreadyExists`]
    /// or another error based on the situation. See [`OpenOptions::open`] for a
    /// non-exhaustive list of likely errors.
    ///
    /// This option is useful because it is atomic. Otherwise between checking
    /// whether a file exists and creating a new one, the file may have been
    /// created by another process (a TOCTOU race condition / attack).
    ///
    /// If `.create_new(true)` is set, [`.create()`] and [`.truncate()`] are
    /// ignored.
    ///
    /// The file must be opened with write or append access in order to create
    /// a new file.
    ///
    /// [`.create()`]: OpenOptions::create
    /// [`.truncate()`]: OpenOptions::truncate
    /// [`AlreadyExists`]: io::ErrorKind::AlreadyExists
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true)
    ///                              .create_new(true)
    ///                              .open("foo.txt");
    /// ```
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }
}

#[cfg(unix)]
impl std::os::unix::fs::OpenOptionsExt for OpenOptions {
    fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags.replace(flags);
        self
    }

    fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode.replace(mode);
        self
    }
}

impl OpenOptions {
    #[cfg(target_os = "android")]
    fn android_mode(&self) -> String {
        let mut mode = String::new();

        if self.read {
            mode.push('r');
        }
        if self.write {
            mode.push('w');
        }
        if self.truncate {
            mode.push('t');
        }
        if self.append {
            mode.push('a');
        }

        mode
    }
}

impl<R: Runtime> Fs<R> {
    pub fn read_to_string<P: Into<FilePath>>(&self, path: P) -> std::io::Result<String> {
        let mut s = String::new();
        self.open(
            path,
            OpenOptions {
                read: true,
                ..Default::default()
            },
        )?
        .read_to_string(&mut s)?;
        Ok(s)
    }

    pub fn read<P: Into<FilePath>>(&self, path: P) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.open(
            path,
            OpenOptions {
                read: true,
                ..Default::default()
            },
        )?
        .read_to_end(&mut buf)?;
        Ok(buf)
    }
}

// implement ScopeObject here instead of in the scope module because it is also used on the build script
// and we don't want to add tauri as a build dependency
impl ScopeObject for scope::Entry {
    type Error = Error;
    fn deserialize<R: Runtime>(
        app: &AppHandle<R>,
        raw: Value,
    ) -> std::result::Result<Self, Self::Error> {
        let path = serde_json::from_value(raw.into()).map(|raw| match raw {
            scope::EntryRaw::Value(path) => path,
            scope::EntryRaw::Object { path } => path,
        })?;

        match app.path().parse(path) {
            Ok(path) => Ok(Self { path: Some(path) }),
            #[cfg(not(target_os = "android"))]
            Err(tauri::Error::UnknownPath) => Ok(Self { path: None }),
            Err(err) => Err(err.into()),
        }
    }
}

pub(crate) struct Scope {
    pub(crate) scope: tauri::fs::Scope,
    pub(crate) require_literal_leading_dot: Option<bool>,
}

pub trait FsExt<R: Runtime> {
    fn fs_scope(&self) -> tauri::fs::Scope;
    fn try_fs_scope(&self) -> Option<tauri::fs::Scope>;

    /// Cross platform file system APIs that also support manipulating Android files.
    fn fs(&self) -> &Fs<R>;
}

impl<R: Runtime, T: Manager<R>> FsExt<R> for T {
    fn fs_scope(&self) -> tauri::fs::Scope {
        self.state::<Scope>().scope.clone()
    }

    fn try_fs_scope(&self) -> Option<tauri::fs::Scope> {
        self.try_state::<Scope>().map(|s| s.scope.clone())
    }

    fn fs(&self) -> &Fs<R> {
        self.state::<Fs<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R, Option<config::Config>> {
    PluginBuilder::<R, Option<config::Config>>::new("fs")
        .invoke_handler(tauri::generate_handler![
            commands::create,
            commands::open,
            commands::copy_file,
            commands::close,
            commands::mkdir,
            commands::read_dir,
            commands::read,
            commands::read_file,
            commands::read_text_file,
            commands::read_text_file_lines,
            commands::read_text_file_lines_next,
            commands::remove,
            commands::rename,
            commands::seek,
            commands::stat,
            commands::lstat,
            commands::fstat,
            commands::truncate,
            commands::ftruncate,
            commands::write,
            commands::write_file,
            commands::write_text_file,
            commands::exists,
            #[cfg(feature = "watch")]
            watcher::watch,
            #[cfg(feature = "watch")]
            watcher::unwatch
        ])
        .setup(|app, api| {
            let scope = Scope {
                require_literal_leading_dot: api
                    .config()
                    .as_ref()
                    .and_then(|c| c.require_literal_leading_dot),
                scope: tauri::fs::Scope::new(app, &FsScope::default())?,
            };

            #[cfg(target_os = "android")]
            {
                let fs = mobile::init(app, api)?;
                app.manage(fs);
            }
            #[cfg(not(target_os = "android"))]
            app.manage(Fs(app.clone()));

            app.manage(scope);
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::WindowEvent {
                label: _,
                event: WindowEvent::DragDrop(DragDropEvent::Drop { paths, position: _ }),
                ..
            } = event
            {
                let scope = app.fs_scope();
                for path in paths {
                    if path.is_file() {
                        let _ = scope.allow_file(path);
                    } else {
                        let _ = scope.allow_directory(path, true);
                    }
                }
            }
        })
        .build()
}
