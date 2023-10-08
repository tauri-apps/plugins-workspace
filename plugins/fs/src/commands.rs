// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Scope;
use anyhow::Context;
use serde::{Deserialize, Serialize, Serializer};
use tauri::{
    path::{BaseDirectory, SafePathBuf},
    Manager, Runtime, Window,
};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::{
    fs::{self, symlink_metadata, File},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{Error, FsExt, Result};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Plugin(#[from] Error),
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

type CommandResult<T> = std::result::Result<T, CommandError>;

/// The options for the directory functions on the file system API.
#[derive(Debug, Clone, Deserialize)]
pub struct DirOperationOptions {
    /// Whether the API should recursively perform the operation on the directory.
    #[serde(default)]
    pub recursive: bool,
    /// The base directory of the operation.
    /// The directory path of the BaseDirectory will be the prefix of the defined directory path.
    pub dir: Option<BaseDirectory>,
}

/// The options for the file functions on the file system API.
#[derive(Debug, Clone, Deserialize)]
pub struct FileOperationOptions {
    /// The base directory of the operation.
    /// The directory path of the BaseDirectory will be the prefix of the defined file path.
    pub dir: Option<BaseDirectory>,
}

fn resolve_path<R: Runtime>(
    window: &Window<R>,
    path: SafePathBuf,
    dir: Option<BaseDirectory>,
) -> Result<PathBuf> {
    let path = if let Some(dir) = dir {
        window
            .path()
            .resolve(&path, dir)
            .map_err(Error::CannotResolvePath)?
    } else {
        path.as_ref().to_path_buf()
    };
    if window.fs_scope().is_allowed(&path) {
        Ok(path)
    } else {
        Err(Error::PathForbidden(path))
    }
}

#[tauri::command]
pub fn read_file<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    options: Option<FileOperationOptions>,
) -> CommandResult<Vec<u8>> {
    let resolved_path = resolve_path(&window, path, options.and_then(|o| o.dir))?;
    fs::read(&resolved_path)
        .with_context(|| format!("path: {}", resolved_path.display()))
        .map_err(Into::into)
}

#[tauri::command]
pub fn read_text_file<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    options: Option<FileOperationOptions>,
) -> CommandResult<String> {
    let resolved_path = resolve_path(&window, path, options.and_then(|o| o.dir))?;
    fs::read_to_string(&resolved_path)
        .with_context(|| format!("path: {}", resolved_path.display()))
        .map_err(Into::into)
}

#[tauri::command]
pub fn write_file<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    contents: Vec<u8>,
    options: Option<FileOperationOptions>,
) -> CommandResult<()> {
    let resolved_path = resolve_path(&window, path, options.and_then(|o| o.dir))?;
    File::create(&resolved_path)
        .with_context(|| format!("path: {}", resolved_path.display()))
        .map_err(Into::into)
        .and_then(|mut f| {
            f.write_all(&contents).map_err(|err| anyhow::anyhow!("{}", err))?;
            f.set_len(contents.len() as u64).map_err(|err| anyhow::anyhow!("{}", err))?;
            Ok(())
        })
}

#[derive(Clone, Copy)]
struct ReadDirOptions<'a> {
    pub scope: Option<&'a Scope>,
}

#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct DiskEntry {
    /// The path to the entry.
    pub path: PathBuf,
    /// The name of the entry (file name with extension or directory name).
    pub name: Option<String>,
    /// The children of this entry if it's a directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DiskEntry>>,
}

fn read_dir_with_options<P: AsRef<Path>>(
    path: P,
    recursive: bool,
    options: ReadDirOptions<'_>,
) -> Result<Vec<DiskEntry>> {
    let mut files_and_dirs: Vec<DiskEntry> = vec![];
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        let path_as_string = path.display().to_string();

        if let Ok(flag) = path.metadata().map(|m| m.is_dir()) {
            let is_symlink = symlink_metadata(&path).map(|md| md.is_symlink())?;
            files_and_dirs.push(DiskEntry {
                path: path.clone(),
                children: if flag {
                    Some(
                        if recursive
                            && (!is_symlink
                                || options.scope.map(|s| s.is_allowed(&path)).unwrap_or(true))
                        {
                            read_dir_with_options(&path_as_string, true, options)?
                        } else {
                            vec![]
                        },
                    )
                } else {
                    None
                },
                name: path
                    .file_name()
                    .map(|name| name.to_string_lossy())
                    .map(|name| name.to_string()),
            });
        }
    }
    Result::Ok(files_and_dirs)
}

#[tauri::command]
pub fn read_dir<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
) -> CommandResult<Vec<DiskEntry>> {
    let (recursive, dir) = if let Some(options_value) = options {
        (options_value.recursive, options_value.dir)
    } else {
        (false, None)
    };
    let resolved_path = resolve_path(&window, path, dir)?;
    read_dir_with_options(
        &resolved_path,
        recursive,
        ReadDirOptions {
            scope: Some(window.fs_scope()),
        },
    )
    .with_context(|| format!("path: {}", resolved_path.display()))
    .map_err(Into::into)
}

#[tauri::command]
pub fn copy_file<R: Runtime>(
    window: Window<R>,
    source: SafePathBuf,
    destination: SafePathBuf,
    options: Option<FileOperationOptions>,
) -> CommandResult<()> {
    match options.and_then(|o| o.dir) {
        Some(dir) => {
            let src = resolve_path(&window, source, Some(dir))?;
            let dest = resolve_path(&window, destination, Some(dir))?;
            fs::copy(&src, &dest)
                .with_context(|| format!("source: {}, dest: {}", src.display(), dest.display()))?
        }
        None => fs::copy(&source, &destination).with_context(|| {
            format!(
                "source: {}, dest: {}",
                source.display(),
                destination.display()
            )
        })?,
    };
    Ok(())
}

#[tauri::command]
pub fn create_dir<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
) -> CommandResult<()> {
    let (recursive, dir) = if let Some(options_value) = options {
        (options_value.recursive, options_value.dir)
    } else {
        (false, None)
    };
    let resolved_path = resolve_path(&window, path, dir)?;
    if recursive {
        fs::create_dir_all(&resolved_path)
            .with_context(|| format!("path: {}", resolved_path.display()))?;
    } else {
        fs::create_dir(&resolved_path)
            .with_context(|| format!("path: {} (non recursive)", resolved_path.display()))?;
    }

    Ok(())
}

#[tauri::command]
pub fn remove_dir<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
) -> CommandResult<()> {
    let (recursive, dir) = if let Some(options_value) = options {
        (options_value.recursive, options_value.dir)
    } else {
        (false, None)
    };
    let resolved_path = resolve_path(&window, path, dir)?;
    if recursive {
        fs::remove_dir_all(&resolved_path)
            .with_context(|| format!("path: {}", resolved_path.display()))?;
    } else {
        fs::remove_dir(&resolved_path)
            .with_context(|| format!("path: {} (non recursive)", resolved_path.display()))?;
    }

    Ok(())
}

#[tauri::command]
pub fn remove_file<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    options: Option<FileOperationOptions>,
) -> CommandResult<()> {
    let resolved_path = resolve_path(&window, path, options.and_then(|o| o.dir))?;
    fs::remove_file(&resolved_path)
        .with_context(|| format!("path: {}", resolved_path.display()))?;
    Ok(())
}

#[tauri::command]
pub fn rename_file<R: Runtime>(
    window: Window<R>,
    old_path: SafePathBuf,
    new_path: SafePathBuf,
    options: Option<FileOperationOptions>,
) -> CommandResult<()> {
    match options.and_then(|o| o.dir) {
        Some(dir) => {
            let old = resolve_path(&window, old_path, Some(dir))?;
            let new = resolve_path(&window, new_path, Some(dir))?;
            fs::rename(&old, &new)
                .with_context(|| format!("old: {}, new: {}", old.display(), new.display()))?
        }
        None => fs::rename(&old_path, &new_path)
            .with_context(|| format!("old: {}, new: {}", old_path.display(), new_path.display()))?,
    }
    Ok(())
}

#[tauri::command]
pub fn exists<R: Runtime>(
    window: Window<R>,
    path: SafePathBuf,
    options: Option<FileOperationOptions>,
) -> CommandResult<bool> {
    let resolved_path = resolve_path(&window, path, options.and_then(|o| o.dir))?;
    Ok(resolved_path.exists())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
}

#[cfg(unix)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnixMetadata {
    dev: u64,
    ino: u64,
    mode: u32,
    nlink: u64,
    uid: u32,
    gid: u32,
    rdev: u64,
    blksize: u64,
    blocks: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    accessed_at_ms: u64,
    created_at_ms: u64,
    modified_at_ms: u64,
    is_dir: bool,
    is_file: bool,
    is_symlink: bool,
    size: u64,
    permissions: Permissions,
    #[cfg(unix)]
    #[serde(flatten)]
    unix: UnixMetadata,
    #[cfg(windows)]
    file_attributes: u32,
}

fn system_time_to_ms(time: std::io::Result<SystemTime>) -> u64 {
    time.map(|t| {
        let duration_since_epoch = t.duration_since(UNIX_EPOCH).unwrap();
        duration_since_epoch.as_millis() as u64
    })
    .unwrap_or_default()
}

#[tauri::command]
pub async fn metadata(path: PathBuf) -> Result<Metadata> {
    let metadata = std::fs::metadata(path)?;
    let file_type = metadata.file_type();
    let permissions = metadata.permissions();
    Ok(Metadata {
        accessed_at_ms: system_time_to_ms(metadata.accessed()),
        created_at_ms: system_time_to_ms(metadata.created()),
        modified_at_ms: system_time_to_ms(metadata.modified()),
        is_dir: file_type.is_dir(),
        is_file: file_type.is_file(),
        is_symlink: file_type.is_symlink(),
        size: metadata.len(),
        permissions: Permissions {
            readonly: permissions.readonly(),
            #[cfg(unix)]
            mode: permissions.mode(),
        },
        #[cfg(unix)]
        unix: UnixMetadata {
            dev: metadata.dev(),
            ino: metadata.ino(),
            mode: metadata.mode(),
            nlink: metadata.nlink(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            rdev: metadata.rdev(),
            blksize: metadata.blksize(),
            blocks: metadata.blocks(),
        },
        #[cfg(windows)]
        file_attributes: metadata.file_attributes(),
    })
}
