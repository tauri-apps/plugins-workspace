// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Save filesystem and asset scopes and restore them when the app is reopened.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tauri_plugin_fs::FsExt;

use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

// Using 2 separate files so that we don't have to think about write conflicts and not break backwards compat.
const SCOPE_STATE_FILENAME: &str = ".persisted-scope";
#[cfg(feature = "protocol-asset")]
const ASSET_SCOPE_STATE_FILENAME: &str = ".persisted-scope-asset";

// Most of these patterns are just added to try to fix broken files in the wild.
// After a while we can hopefully reduce it to something like [r"[?]", r"[*]", r"\\?\\\?\"]
const PATTERNS: &[&str] = &[
    r"[[]",
    r"[]]",
    r"[?]",
    r"[*]",
    r"\?\?",
    r"\\?\\?\",
    r"\\?\\\?\",
];
const REPLACE_WITH: &[&str] = &[r"[", r"]", r"?", r"*", r"\?", r"\\?\", r"\\?\"];

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Bincode(#[from] Box<bincode::ErrorKind>),
}

#[derive(Debug, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
enum TargetType {
    #[default]
    File,
    Directory,
    RecursiveDirectory,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Scope {
    allowed_paths: Vec<String>,
    forbidden_patterns: Vec<String>,
}

fn fix_pattern(ac: &AhoCorasick, s: &str) -> String {
    let s = ac.replace_all(s, REPLACE_WITH);

    if ac.find(&s).is_some() {
        return fix_pattern(ac, &s);
    }

    s
}

const RESURSIVE_DIRECTORY_SUFFIX: &str = "**";
const DIRECTORY_SUFFIX: &str = "*";

fn detect_scope_type(scope_state_path: &str) -> TargetType {
    if scope_state_path.ends_with(RESURSIVE_DIRECTORY_SUFFIX) {
        TargetType::RecursiveDirectory
    } else if scope_state_path.ends_with(DIRECTORY_SUFFIX) {
        TargetType::Directory
    } else {
        TargetType::File
    }
}

fn fix_directory(path_str: &str) -> &Path {
    let mut path = Path::new(path_str);

    if path.ends_with(DIRECTORY_SUFFIX) || path.ends_with(RESURSIVE_DIRECTORY_SUFFIX) {
        path = match path.parent() {
            Some(value) => value,
            None => return path,
        };
    }

    path
}

fn allow_path(scope: &tauri::fs::Scope, path: &str) {
    let target_type = detect_scope_type(path);

    match target_type {
        TargetType::File => {
            let _ = scope.allow_file(Path::new(path));
        }
        TargetType::Directory => {
            // We remove the '*' at the end of it, else it will be escaped by the pattern.
            let _ = scope.allow_directory(fix_directory(path), false);
        }
        TargetType::RecursiveDirectory => {
            // We remove the '**' at the end of it, else it will be escaped by the pattern.
            let _ = scope.allow_directory(fix_directory(path), true);
        }
    }
}

fn forbid_path(scope: &tauri::fs::Scope, path: &str) {
    let target_type = detect_scope_type(path);

    match target_type {
        TargetType::File => {
            let _ = scope.forbid_file(Path::new(path));
        }
        TargetType::Directory => {
            let _ = scope.forbid_directory(fix_directory(path), false);
        }
        TargetType::RecursiveDirectory => {
            let _ = scope.forbid_directory(fix_directory(path), true);
        }
    }
}

fn save_scopes(scope: &tauri::fs::Scope, app_dir: &Path, scope_state_path: &Path) {
    let scope = Scope {
        allowed_paths: scope
            .allowed_patterns()
            .into_iter()
            .map(|p| p.to_string())
            .collect(),
        forbidden_patterns: scope
            .forbidden_patterns()
            .into_iter()
            .map(|p| p.to_string())
            .collect(),
    };

    let _ = create_dir_all(app_dir)
        .and_then(|_| File::create(scope_state_path))
        .map_err(Error::Io)
        .and_then(|mut f| {
            f.write_all(&bincode::serialize(&scope).map_err(Error::from)?)
                .map_err(Into::into)
        });
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("persisted-scope")
        .setup(|app, _api| {
            let fs_scope = app.try_fs_scope();
            #[cfg(feature = "protocol-asset")]
            let asset_protocol_scope = app.asset_protocol_scope();
            let app = app.clone();
            let app_dir = app.path().app_data_dir();

            if let Ok(app_dir) = app_dir {
                let fs_scope_state_path = app_dir.join(SCOPE_STATE_FILENAME);
                #[cfg(feature = "protocol-asset")]
                let asset_scope_state_path = app_dir.join(ASSET_SCOPE_STATE_FILENAME);

                if let Some(fs_scope) = &fs_scope {
                     let _ = fs_scope.forbid_file(&fs_scope_state_path);
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("Please make sure to register the `fs` plugin before the `persisted-scope` plugin!");
                }
                #[cfg(feature = "protocol-asset")]
                let _ = asset_protocol_scope.forbid_file(&asset_scope_state_path);

                // We're trying to fix broken .persisted-scope files seamlessly, so we'll be running this on the values read on the saved file.
                // We will still save some semi-broken values because the scope events are quite spammy and we don't want to reduce runtime performance any further.
                let ac = AhoCorasick::new(PATTERNS).unwrap(/* This should be impossible to fail since we're using a small static input */);

                if let Some(fs_scope) = &fs_scope {
                    if fs_scope_state_path.exists() {
                    let scope: Scope = std::fs::read(&fs_scope_state_path)
                        .map_err(Error::from)
                        .and_then(|scope| bincode::deserialize(&scope).map_err(Into::into))
                        .unwrap_or_default();

                    for allowed in &scope.allowed_paths {
                        let allowed = fix_pattern(&ac, allowed);
                        allow_path(fs_scope, &allowed);
                    }
                    for forbidden in &scope.forbidden_patterns {
                        let forbidden = fix_pattern(&ac, forbidden);
                        forbid_path(fs_scope, &forbidden);
                    }

                    // Manually save the fixed scopes to disk once.
                    // This is needed to fix broken .peristed-scope files in case the app doesn't update the scope itself.
                    save_scopes(fs_scope, &app_dir, &fs_scope_state_path);
                }
            }

                #[cfg(feature = "protocol-asset")]
                if asset_scope_state_path.exists() {
                    let scope: Scope = std::fs::read(&asset_scope_state_path)
                        .map_err(Error::from)
                        .and_then(|scope| bincode::deserialize(&scope).map_err(Into::into))
                        .unwrap_or_default();

                    for allowed in &scope.allowed_paths {
                        let allowed = fix_pattern(&ac, allowed);
                        allow_path(&asset_protocol_scope, &allowed);
                    }
                    for forbidden in &scope.forbidden_patterns {
                        let forbidden = fix_pattern(&ac, forbidden);
                        forbid_path(&asset_protocol_scope, &forbidden);
                    }

                    // Manually save the fixed scopes to disk once.
                    save_scopes(&asset_protocol_scope, &app_dir, &asset_scope_state_path);
                }

                #[cfg(feature = "protocol-asset")]
                let app_dir_ = app_dir.clone();

                if let Some(fs_scope) = &fs_scope {
                    let app_ = app.clone();
                    fs_scope.listen(move |event| {
                        if let tauri::fs::Event::PathAllowed(_) = event {
                            save_scopes(&app_.fs_scope(), &app_dir, &fs_scope_state_path);
                        }
                    });
                }

                #[cfg(feature = "protocol-asset")]
                {
                    let asset_protocol_scope_ = asset_protocol_scope.clone();
                    asset_protocol_scope.listen(move |event| {
                        if let tauri::scope::fs::Event::PathAllowed(_) = event {
                            save_scopes(&asset_protocol_scope_, &app_dir_, &asset_scope_state_path);
                        }
                    });
                }
            }
            Ok(())
        })
        .build()
}
