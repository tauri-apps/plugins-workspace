// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Access the file system.
 *
 * ## Security
 *
 * This module prevents path traversal, not allowing absolute paths or parent dir components
 * (i.e. "/usr/path/to/file" or "../path/to/file" paths are not allowed).
 * Paths accessed with this API must be relative to one of the {@link BaseDirectory | base directories}
 * so if you need access to arbitrary filesystem paths, you must write such logic on the core layer instead.
 *
 * The API has a scope configuration that forces you to restrict the paths that can be accessed using glob patterns.
 *
 * The scope configuration is an array of glob patterns describing folder paths that are allowed.
 * For instance, this scope configuration only allows accessing files on the
 * *databases* folder of the {@link path.appDataDir | $APPDATA directory}:
 * ```json
 * {
 *   "plugins": {
 *     "fs": {
 *       "scope": ["$APPDATA/databases/*"]
 *     }
 *   }
 * }
 * ```
 *
 * Notice the use of the `$APPDATA` variable. The value is injected at runtime, resolving to the {@link path.appDataDir | app data directory}.
 * The available variables are:
 * {@link path.appConfigDir | `$APPCONFIG`}, {@link path.appDataDir | `$APPDATA`}, {@link path.appLocalDataDir | `$APPLOCALDATA`},
 * {@link path.appCacheDir | `$APPCACHE`}, {@link path.appLogDir | `$APPLOG`},
 * {@link path.audioDir | `$AUDIO`}, {@link path.cacheDir | `$CACHE`}, {@link path.configDir | `$CONFIG`}, {@link path.dataDir | `$DATA`},
 * {@link path.localDataDir | `$LOCALDATA`}, {@link path.desktopDir | `$DESKTOP`}, {@link path.documentDir | `$DOCUMENT`},
 * {@link path.downloadDir | `$DOWNLOAD`}, {@link path.executableDir | `$EXE`}, {@link path.fontDir | `$FONT`}, {@link path.homeDir | `$HOME`},
 * {@link path.pictureDir | `$PICTURE`}, {@link path.publicDir | `$PUBLIC`}, {@link path.runtimeDir | `$RUNTIME`},
 * {@link path.templateDir | `$TEMPLATE`}, {@link path.videoDir | `$VIDEO`}, {@link path.resourceDir | `$RESOURCE`},
 * {@link os.tempdir | `$TEMP`}.
 *
 * Trying to execute any API with a URL not configured on the scope results in a promise rejection due to denied access.
 *
 * Note that this scope applies to **all** APIs on this module.
 *
 * @module
 */

import { BaseDirectory } from "@tauri-apps/api/path";

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

interface Permissions {
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
interface Metadata {
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

/**
 * @since 1.0.0
 */
interface FsOptions {
  dir?: BaseDirectory;
  // note that adding fields here needs a change in the writeBinaryFile check
}

/**
 * @since 1.0.0
 */
interface FsDirOptions {
  dir?: BaseDirectory;
  recursive?: boolean;
}

/**
 * Options object used to write a UTF-8 string to a file.
 *
 * @since 1.0.0
 */
interface FsTextFileOption {
  /** Path to the file to write. */
  path: string;
  /** The UTF-8 string to write to the file. */
  contents: string;
}

type BinaryFileContents = Iterable<number> | ArrayLike<number> | ArrayBuffer;

/**
 * Options object used to write a binary data to a file.
 *
 * @since 1.0.0
 */
interface FsBinaryFileOption {
  /** Path to the file to write. */
  path: string;
  /** The byte array contents. */
  contents: BinaryFileContents;
}

/**
 * @since 1.0.0
 */
interface FileEntry {
  path: string;
  /**
   * Name of the directory/file
   * can be null if the path terminates with `..`
   */
  name?: string;
  /** Children of this entry if it's a directory; null otherwise */
  children?: FileEntry[];
}

/**
 * Reads a file as an UTF-8 encoded string.
 * @example
 * ```typescript
 * import { readTextFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Read the text file in the `$APPCONFIG/app.conf` path
 * const contents = await readTextFile('app.conf', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @since 1.0.0
 */
async function readTextFile(
  filePath: string,
  options: FsOptions = {}
): Promise<string> {
  return await window.__TAURI_INVOKE__("plugin:fs|read_text_file", {
    path: filePath,
    options,
  });
}

/**
 * Reads a file as byte array.
 * @example
 * ```typescript
 * import { readBinaryFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Read the image file in the `$RESOURCEDIR/avatar.png` path
 * const contents = await readBinaryFile('avatar.png', { dir: BaseDirectory.Resource });
 * ```
 *
 * @since 1.0.0
 */
async function readBinaryFile(
  filePath: string,
  options: FsOptions = {}
): Promise<Uint8Array> {
  const arr = await window.__TAURI_INVOKE__<number[]>("plugin:fs|read_file", {
    path: filePath,
    options,
  });

  return Uint8Array.from(arr);
}

/**
 * Writes a UTF-8 text file.
 * @example
 * ```typescript
 * import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Write a text file to the `$APPCONFIG/app.conf` path
 * await writeTextFile('app.conf', 'file contents', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @since 1.0.0
 */
async function writeTextFile(
  path: string,
  contents: string,
  options?: FsOptions
): Promise<void>;

/**
 * Writes a UTF-8 text file.
 * @example
 * ```typescript
 * import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Write a text file to the `$APPCONFIG/app.conf` path
 * await writeTextFile({ path: 'app.conf', contents: 'file contents' }, { dir: BaseDirectory.AppConfig });
 * ```
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function writeTextFile(
  file: FsTextFileOption,
  options?: FsOptions
): Promise<void>;

/**
 * Writes a UTF-8 text file.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function writeTextFile(
  path: string | FsTextFileOption,
  contents?: string | FsOptions,
  options?: FsOptions
): Promise<void> {
  if (typeof options === "object") {
    Object.freeze(options);
  }
  if (typeof path === "object") {
    Object.freeze(path);
  }

  const file: FsTextFileOption = { path: "", contents: "" };
  let fileOptions: FsOptions | undefined = options;
  if (typeof path === "string") {
    file.path = path;
  } else {
    file.path = path.path;
    file.contents = path.contents;
  }

  if (typeof contents === "string") {
    file.contents = contents ?? "";
  } else {
    fileOptions = contents;
  }

  return await window.__TAURI_INVOKE__("plugin:fs|write_file", {
    path: file.path,
    contents: Array.from(new TextEncoder().encode(file.contents)),
    options: fileOptions,
  });
}

/**
 * Writes a byte array content to a file.
 * @example
 * ```typescript
 * import { writeBinaryFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Write a binary file to the `$APPDATA/avatar.png` path
 * await writeBinaryFile('avatar.png', new Uint8Array([]), { dir: BaseDirectory.AppData });
 * ```
 *
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function writeBinaryFile(
  path: string,
  contents: BinaryFileContents,
  options?: FsOptions
): Promise<void>;

/**
 * Writes a byte array content to a file.
 * @example
 * ```typescript
 * import { writeBinaryFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Write a binary file to the `$APPDATA/avatar.png` path
 * await writeBinaryFile({ path: 'avatar.png', contents: new Uint8Array([]) }, { dir: BaseDirectory.AppData });
 * ```
 *
 * @param file The object containing the file path and contents.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function writeBinaryFile(
  file: FsBinaryFileOption,
  options?: FsOptions
): Promise<void>;

/**
 * Writes a byte array content to a file.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function writeBinaryFile(
  path: string | FsBinaryFileOption,
  contents?: BinaryFileContents | FsOptions,
  options?: FsOptions
): Promise<void> {
  if (typeof options === "object") {
    Object.freeze(options);
  }
  if (typeof path === "object") {
    Object.freeze(path);
  }

  const file: FsBinaryFileOption = { path: "", contents: [] };
  let fileOptions: FsOptions | undefined = options;
  if (typeof path === "string") {
    file.path = path;
  } else {
    file.path = path.path;
    file.contents = path.contents;
  }

  if (contents && "dir" in contents) {
    fileOptions = contents;
  } else if (typeof path === "string") {
    // @ts-expect-error in this case `contents` is always a BinaryFileContents
    file.contents = contents ?? [];
  }

  return await window.__TAURI_INVOKE__("plugin:fs|write_binary_file", {
    path: file.path,
    contents: Array.from(
      file.contents instanceof ArrayBuffer
        ? new Uint8Array(file.contents)
        : file.contents
    ),
    options: fileOptions,
  });
}

/**
 * List directory files.
 * @example
 * ```typescript
 * import { readDir, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Reads the `$APPDATA/users` directory recursively
 * const entries = await readDir('users', { dir: BaseDirectory.AppData, recursive: true });
 *
 * function processEntries(entries) {
 *   for (const entry of entries) {
 *     console.log(`Entry: ${entry.path}`);
 *     if (entry.children) {
 *       processEntries(entry.children)
 *     }
 *   }
 * }
 * ```
 *
 * @since 1.0.0
 */
async function readDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<FileEntry[]> {
  return await window.__TAURI_INVOKE__("plugin:fs|read_dir", {
    path: dir,
    options,
  });
}

/**
 * Creates a directory.
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { createDir, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Create the `$APPDATA/users` directory
 * await createDir('users', { dir: BaseDirectory.AppData, recursive: true });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function createDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<void> {
  return await window.__TAURI_INVOKE__("plugin:fs|create_dir", {
    path: dir,
    options,
  });
}

/**
 * Removes a directory.
 * If the directory is not empty and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { removeDir, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Remove the directory `$APPDATA/users`
 * await removeDir('users', { dir: BaseDirectory.AppData });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function removeDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<void> {
  return await window.__TAURI_INVOKE__("plugin:fs|remove_dir", {
    path: dir,
    options,
  });
}

/**
 * Copies a file to a destination.
 * @example
 * ```typescript
 * import { copyFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Copy the `$APPCONFIG/app.conf` file to `$APPCONFIG/app.conf.bk`
 * await copyFile('app.conf', 'app.conf.bk', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function copyFile(
  source: string,
  destination: string,
  options: FsOptions = {}
): Promise<void> {
  return await window.__TAURI_INVOKE__("plugin:fs|copy_file", {
    source,
    destination,
    options,
  });
}

/**
 * Removes a file.
 * @example
 * ```typescript
 * import { removeFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Remove the `$APPConfig/app.conf` file
 * await removeFile('app.conf', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function removeFile(
  file: string,
  options: FsOptions = {}
): Promise<void> {
  return await window.__TAURI_INVOKE__("plugin:fs|remove_file", {
    path: file,
    options,
  });
}

/**
 * Renames a file.
 * @example
 * ```typescript
 * import { renameFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Rename the `$APPDATA/avatar.png` file
 * await renameFile('avatar.png', 'deleted.png', { dir: BaseDirectory.AppData });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function renameFile(
  oldPath: string,
  newPath: string,
  options: FsOptions = {}
): Promise<void> {
  return await window.__TAURI_INVOKE__("plugin:fs|rename_file", {
    oldPath,
    newPath,
    options,
  });
}

/**
 * Check if a path exists.
 * @example
 * ```typescript
 * import { exists, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Check if the `$APPDATA/avatar.png` file exists
 * await exists('avatar.png', { dir: BaseDirectory.AppData });
 * ```
 *
 * @since 1.0.0
 */
async function exists(path: string): Promise<boolean> {
  return await window.__TAURI_INVOKE__("plugin:fs|exists", { path });
}

/**
 * Returns the metadata for the given path.
 *
 * @since 1.0.0
 */
async function metadata(path: string): Promise<Metadata> {
  return await window
    .__TAURI_INVOKE__<BackendMetadata>("plugin:fs|metadata", {
      path,
    })
    .then((metadata) => {
      const { accessedAtMs, createdAtMs, modifiedAtMs, ...data } = metadata;
      return {
        accessedAt: new Date(accessedAtMs),
        createdAt: new Date(createdAtMs),
        modifiedAt: new Date(modifiedAtMs),
        ...data,
      };
    });
}

export type {
  FsOptions,
  FsDirOptions,
  FsTextFileOption,
  BinaryFileContents,
  FsBinaryFileOption,
  FileEntry,
  Permissions,
  Metadata,
};

export {
  BaseDirectory,
  BaseDirectory as Dir,
  readTextFile,
  readBinaryFile,
  writeTextFile,
  writeTextFile as writeFile,
  writeBinaryFile,
  readDir,
  createDir,
  removeDir,
  copyFile,
  removeFile,
  renameFile,
  exists,
  metadata,
};
