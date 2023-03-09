// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, TauriPlugin},
    FsScopeEvent, Manager, Runtime,
};

use std::{
    fs::{create_dir_all, File},
    io::Write,
};

const SCOPE_STATE_FILENAME: &str = ".persisted-scope";
const PATTERNS: &[&str] = &["[?]", "[[]", "[]]", "[*]"];
const REPLACE_WITH: &[&str] = &["?", "[", "]", "*"];

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    TauriApi(#[from] tauri::api::Error),
    #[error(transparent)]
    Bincode(#[from] Box<bincode::ErrorKind>),
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Scope {
    allowed_paths: Vec<String>,
    forbidden_patterns: Vec<String>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("persisted-scope")
        .setup(|app| {
            let fs_scope = app.fs_scope();
            #[cfg(feature = "protocol-asset")]
            let asset_protocol_scope = app.asset_protocol_scope();
            let app = app.clone();
            let app_dir = app.path_resolver().app_data_dir();

            if let Some(app_dir) = app_dir {
                let scope_state_path = app_dir.join(SCOPE_STATE_FILENAME);

                // We need to filter out glob pattern paths from the scope configs to not pollute the scope with incorrect paths.
                // We can't plainly filter for `*` because `*` is valid in paths on unix.
                let initial_fs_allowed: Vec<String> = fs_scope
                    .allowed_patterns()
                    .into_iter()
                    .map(|p| p.to_string())
                    .collect();
                #[cfg(feature = "protocol-asset")]
                let initial_asset_allowed: Vec<String> = asset_protocol_scope
                    .allowed_patterns()
                    .into_iter()
                    .map(|p| p.to_string())
                    .collect();
                let initial_fs_forbidden: Vec<String> = fs_scope
                    .forbidden_patterns()
                    .into_iter()
                    .map(|p| p.to_string())
                    .collect();
                #[cfg(feature = "protocol-asset")]
                let initial_asset_forbidden: Vec<String> = asset_protocol_scope
                    .forbidden_patterns()
                    .into_iter()
                    .map(|p| p.to_string())
                    .collect();

                let _ = fs_scope.forbid_file(&scope_state_path);
                #[cfg(feature = "protocol-asset")]
                let _ = asset_protocol_scope.forbid_file(&scope_state_path);

                if scope_state_path.exists() {
                    let scope: Scope = tauri::api::file::read_binary(&scope_state_path)
                        .map_err(Error::from)
                        .and_then(|scope| bincode::deserialize(&scope).map_err(Into::into))
                        .unwrap_or_default();
                    for allowed in &scope.allowed_paths {
                        if !initial_fs_allowed.contains(&allowed) {
                            let _ = fs_scope.allow_file(&allowed);
                        }
                        #[cfg(feature = "protocol-asset")]
                        if !initial_asset_allowed.contains(&allowed) {
                            let _ = asset_protocol_scope.allow_file(&allowed);
                        }
                    }
                    for forbidden in &scope.forbidden_patterns {
                        // forbid the path as is
                        if !initial_fs_forbidden.contains(&forbidden) {
                            let _ = fs_scope.forbid_file(&forbidden);
                        }
                        #[cfg(feature = "protocol-asset")]
                        if !initial_asset_forbidden.contains(&forbidden) {
                            let _ = asset_protocol_scope.forbid_file(&forbidden);
                        }
                    }
                }

                // We could also "fix" the paths on app start if we notice any runtime performance problems.
                let ac = AhoCorasick::new_auto_configured(PATTERNS);

                fs_scope.listen(move |event| {
                    let fs_scope = app.fs_scope();
                    if let FsScopeEvent::PathAllowed(_) = event {
                        let scope = Scope {
                            allowed_paths: fs_scope
                                .allowed_patterns()
                                .into_iter()
                                .map(|p| ac.replace_all(p.as_str(), REPLACE_WITH))
                                .collect(),
                            forbidden_patterns: fs_scope
                                .forbidden_patterns()
                                .into_iter()
                                .map(|p| ac.replace_all(p.as_str(), REPLACE_WITH))
                                .collect(),
                        };

                        let _ = create_dir_all(&app_dir)
                            .and_then(|_| File::create(&scope_state_path))
                            .map_err(Error::Io)
                            .and_then(|mut f| {
                                f.write_all(&bincode::serialize(&scope).map_err(Error::from)?)
                                    .map_err(Into::into)
                            });
                    }
                });
            }
            Ok(())
        })
        .build()
}
