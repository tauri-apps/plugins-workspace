// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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

                let _ = fs_scope.forbid_file(&scope_state_path);
                #[cfg(feature = "protocol-asset")]
                let _ = asset_protocol_scope.forbid_file(&scope_state_path);

                if scope_state_path.exists() {
                    let scope: Scope = tauri::api::file::read_binary(&scope_state_path)
                        .map_err(Error::from)
                        .and_then(|scope| bincode::deserialize(&scope).map_err(Into::into))
                        .unwrap_or_default();
                    for allowed in &scope.allowed_paths {
                        // allows the path as is
                        let _ = fs_scope.allow_file(allowed);
                        #[cfg(feature = "protocol-asset")]
                        let _ = asset_protocol_scope.allow_file(allowed);
                    }
                    for forbidden in &scope.forbidden_patterns {
                        // forbid the path as is
                        let _ = fs_scope.forbid_file(forbidden);
                        #[cfg(feature = "protocol-asset")]
                        let _ = asset_protocol_scope.forbid_file(forbidden);
                    }
                }

                fs_scope.listen(move |event| {
                    let fs_scope = app.fs_scope();
                    if let FsScopeEvent::PathAllowed(_) = event {
                        let scope = Scope {
                            allowed_paths: fs_scope
                                .allowed_patterns()
                                .into_iter()
                                .map(|p| p.to_string())
                                .collect(),
                            forbidden_patterns: fs_scope
                                .forbidden_patterns()
                                .into_iter()
                                .map(|p| p.to_string())
                                .collect(),
                        };
                        let scope_state_path = scope_state_path.clone();

                        let _ = create_dir_all(&app_dir)
                            .and_then(|_| File::create(scope_state_path))
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
