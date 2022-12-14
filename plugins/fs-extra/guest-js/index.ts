// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/tauri";

export interface Permissions {
  /**
   * `true` if these permissions describe a readonly (unwritable) file.
   */
  readonly: boolean;
  /**
   * The underlying raw `st_mode` bits that contain the standard Unix permissions for this file.
   */
  mode: number | undefined;
}

/**
 * Metadata information about a file.
 * This structure is returned from the `metadata` function or method
 * and represents known metadata about a file such as its permissions, size, modification times, etc.
 */
export interface Metadata {
  /**
   * The last access time of this metadata.
   */
  accessedAt: Date;
  /**
   * The creation time listed in this metadata.
   */
  createdAt: Date;
  /**
   * The last modification time listed in this metadata.
   */
  modifiedAt: Date;
  /**
   * `true` if this metadata is for a directory.
   */
  isDir: boolean;
  /**
   * `true` if this metadata is for a regular file.
   */
  isFile: boolean;
  /**
   * `true` if this metadata is for a symbolic link.
   */
  isSymlink: boolean;
  /**
   * The size of the file, in bytes, this metadata is for.
   */
  size: number;
  /**
   * The permissions of the file this metadata is for.
   */
  permissions: Permissions;
  /**
   * The ID of the device containing the file. Only available on Unix.
   */
  dev: number | undefined;
  /**
   * The inode number. Only available on Unix.
   */
  ino: number | undefined;
  /**
   * The rights applied to this file. Only available on Unix.
   */
  mode: number | undefined;
  /**
   * The number of hard links pointing to this file. Only available on Unix.
   */
  nlink: number | undefined;
  /**
   * The user ID of the owner of this file. Only available on Unix.
   */
  uid: number | undefined;
  /**
   * The group ID of the owner of this file. Only available on Unix.
   */
  gid: number | undefined;
  /**
   * The device ID of this file (if it is a special one). Only available on Unix.
   */
  rdev: number | undefined;
  /**
   * The block size for filesystem I/O. Only available on Unix.
   */
  blksize: number | undefined;
  /**
   * The number of blocks allocated to the file, in 512-byte units. Only available on Unix.
   */
  blocks: number | undefined;
}

interface BackendMetadata {
  accessedAtMs: number;
  createdAtMs: number;
  modifiedAtMs: number;
  isDir: boolean;
  isFile: boolean;
  isSymlink: boolean;
  size: number;
  permissions: Permissions;
  dev: number | undefined;
  ino: number | undefined;
  mode: number | undefined;
  nlink: number | undefined;
  uid: number | undefined;
  gid: number | undefined;
  rdev: number | undefined;
  blksize: number | undefined;
  blocks: number | undefined;
}

export async function metadata(path: string): Promise<Metadata> {
  return await invoke<BackendMetadata>("plugin:fs-extra|metadata", {
    path,
  }).then((metadata) => {
    const { accessedAtMs, createdAtMs, modifiedAtMs, ...data } = metadata;
    return {
      accessedAt: new Date(accessedAtMs),
      createdAt: new Date(createdAtMs),
      modifiedAt: new Date(modifiedAtMs),
      ...data,
    };
  });
}

export async function exists(path: string): Promise<boolean> {
  return await invoke("plugin:fs-extra|exists", { path });
}
