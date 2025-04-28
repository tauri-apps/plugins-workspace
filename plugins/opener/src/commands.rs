// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use tauri::{
    ipc::{CommandScope, GlobalScope},
    AppHandle, Runtime,
};

use crate::{scope::Scope, Error, OpenerExt};

#[tauri::command]
pub async fn open_url<R: Runtime>(
    app: AppHandle<R>,
    command_scope: CommandScope<crate::scope::Entry>,
    global_scope: GlobalScope<crate::scope::Entry>,
    url: String,
    with: Option<String>,
) -> crate::Result<()> {
    let scope = Scope::new(
        &app,
        command_scope
            .allows()
            .iter()
            .chain(global_scope.allows())
            .collect(),
        command_scope
            .denies()
            .iter()
            .chain(global_scope.denies())
            .collect(),
    );

    if scope.is_url_allowed(&url, with.as_deref()) {
        app.opener().open_url(url, with)
    } else {
        Err(Error::ForbiddenUrl { url, with })
    }
}

#[tauri::command]
pub async fn open_path<R: Runtime>(
    app: AppHandle<R>,
    command_scope: CommandScope<crate::scope::Entry>,
    global_scope: GlobalScope<crate::scope::Entry>,
    path: String,
    with: Option<String>,
) -> crate::Result<()> {
    let scope = Scope::new(
        &app,
        command_scope
            .allows()
            .iter()
            .chain(global_scope.allows())
            .collect(),
        command_scope
            .denies()
            .iter()
            .chain(global_scope.denies())
            .collect(),
    );

    if scope.is_path_allowed(Path::new(&path), with.as_deref())? {
        app.opener().open_path(path, with)
    } else {
        Err(Error::ForbiddenPath { path, with })
    }
}

#[tauri::command]
pub async fn reveal_item_in_dir(path: PathBuf) -> crate::Result<()> {
    crate::reveal_item_in_dir(path)
}
