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
/**
 * @since 1.0.0
 */
declare enum BaseDirectory {
    Audio = 1,
    Cache = 2,
    Config = 3,
    Data = 4,
    LocalData = 5,
    Document = 6,
    Download = 7,
    Picture = 8,
    Public = 9,
    Video = 10,
    Resource = 11,
    Temp = 12,
    AppConfig = 13,
    AppData = 14,
    AppLocalData = 15,
    AppCache = 16,
    AppLog = 17,
    Desktop = 18,
    Executable = 19,
    Font = 20,
    Home = 21,
    Runtime = 22,
    Template = 23
}
/**
 * @since 1.0.0
 */
interface FsOptions {
    dir?: BaseDirectory;
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
 * import { readTextFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Read the text file in the `$APPCONFIG/app.conf` path
 * const contents = await readTextFile('app.conf', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @since 1.0.0
 */
declare function readTextFile(filePath: string, options?: FsOptions): Promise<string>;
/**
 * Reads a file as byte array.
 * @example
 * ```typescript
 * import { readBinaryFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Read the image file in the `$RESOURCEDIR/avatar.png` path
 * const contents = await readBinaryFile('avatar.png', { dir: BaseDirectory.Resource });
 * ```
 *
 * @since 1.0.0
 */
declare function readBinaryFile(filePath: string, options?: FsOptions): Promise<Uint8Array>;
/**
 * Writes a UTF-8 text file.
 * @example
 * ```typescript
 * import { writeTextFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Write a text file to the `$APPCONFIG/app.conf` path
 * await writeTextFile('app.conf', 'file contents', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @since 1.0.0
 */
declare function writeTextFile(path: string, contents: string, options?: FsOptions): Promise<void>;
/**
 * Writes a UTF-8 text file.
 * @example
 * ```typescript
 * import { writeTextFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Write a text file to the `$APPCONFIG/app.conf` path
 * await writeTextFile({ path: 'app.conf', contents: 'file contents' }, { dir: BaseDirectory.AppConfig });
 * ```
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
declare function writeTextFile(file: FsTextFileOption, options?: FsOptions): Promise<void>;
/**
 * Writes a byte array content to a file.
 * @example
 * ```typescript
 * import { writeBinaryFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Write a binary file to the `$APPDATA/avatar.png` path
 * await writeBinaryFile('avatar.png', new Uint8Array([]), { dir: BaseDirectory.AppData });
 * ```
 *
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
declare function writeBinaryFile(path: string, contents: BinaryFileContents, options?: FsOptions): Promise<void>;
/**
 * Writes a byte array content to a file.
 * @example
 * ```typescript
 * import { writeBinaryFile, BaseDirectory } from '@tauri-apps/api/fs';
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
declare function writeBinaryFile(file: FsBinaryFileOption, options?: FsOptions): Promise<void>;
/**
 * List directory files.
 * @example
 * ```typescript
 * import { readDir, BaseDirectory } from '@tauri-apps/api/fs';
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
declare function readDir(dir: string, options?: FsDirOptions): Promise<FileEntry[]>;
/**
 * Creates a directory.
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { createDir, BaseDirectory } from '@tauri-apps/api/fs';
 * // Create the `$APPDATA/users` directory
 * await createDir('users', { dir: BaseDirectory.AppData, recursive: true });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
declare function createDir(dir: string, options?: FsDirOptions): Promise<void>;
/**
 * Removes a directory.
 * If the directory is not empty and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { removeDir, BaseDirectory } from '@tauri-apps/api/fs';
 * // Remove the directory `$APPDATA/users`
 * await removeDir('users', { dir: BaseDirectory.AppData });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
declare function removeDir(dir: string, options?: FsDirOptions): Promise<void>;
/**
 * Copies a file to a destination.
 * @example
 * ```typescript
 * import { copyFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Copy the `$APPCONFIG/app.conf` file to `$APPCONFIG/app.conf.bk`
 * await copyFile('app.conf', 'app.conf.bk', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
declare function copyFile(source: string, destination: string, options?: FsOptions): Promise<void>;
/**
 * Removes a file.
 * @example
 * ```typescript
 * import { removeFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Remove the `$APPConfig/app.conf` file
 * await removeFile('app.conf', { dir: BaseDirectory.AppConfig });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
declare function removeFile(file: string, options?: FsOptions): Promise<void>;
/**
 * Renames a file.
 * @example
 * ```typescript
 * import { renameFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Rename the `$APPDATA/avatar.png` file
 * await renameFile('avatar.png', 'deleted.png', { dir: BaseDirectory.AppData });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
declare function renameFile(oldPath: string, newPath: string, options?: FsOptions): Promise<void>;
/**
 * Check if a path exists.
 * @example
 * ```typescript
 * import { exists, BaseDirectory } from '@tauri-apps/api/fs';
 * // Check if the `$APPDATA/avatar.png` file exists
 * await exists('avatar.png', { dir: BaseDirectory.AppData });
 * ```
 *
 * @since 1.0.0
 */
declare function exists(path: string): Promise<boolean>;
/**
 * Returns the metadata for the given path.
 *
 * @since 1.0.0
 */
declare function metadata(path: string): Promise<Metadata>;
export type { FsOptions, FsDirOptions, FsTextFileOption, BinaryFileContents, FsBinaryFileOption, FileEntry, Permissions, Metadata, };
export { BaseDirectory, BaseDirectory as Dir, readTextFile, readBinaryFile, writeTextFile, writeTextFile as writeFile, writeBinaryFile, readDir, createDir, removeDir, copyFile, removeFile, renameFile, exists, metadata };
