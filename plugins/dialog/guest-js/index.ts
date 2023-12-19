// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";

interface FileResponse {
  base64Data?: string;
  duration?: number;
  height?: number;
  width?: number;
  mimeType?: string;
  modifiedAt?: number;
  name?: string;
  path: string;
  size: number;
}

/**
 * Extension filters for the file dialog.
 *
 * @since 2.0.0
 */
interface DialogFilter {
  /** Filter name. */
  name: string;
  /**
   * Extensions to filter, without a `.` prefix.
   * @example
   * ```typescript
   * extensions: ['svg', 'png']
   * ```
   */
  extensions: string[];
}

/**
 * Options for the open dialog.
 *
 * @since 2.0.0
 */
interface OpenDialogOptions {
  /** The title of the dialog window. */
  title?: string;
  /** The filters of the dialog. */
  filters?: DialogFilter[];
  /** Initial directory or file path. */
  defaultPath?: string;
  /** Whether the dialog allows multiple selection or not. */
  multiple?: boolean;
  /** Whether the dialog is a directory selection or not. */
  directory?: boolean;
  /**
   * If `directory` is true, indicates that it will be read recursively later.
   * Defines whether subdirectories will be allowed on the scope or not.
   */
  recursive?: boolean;
}

/**
 * Options for the save dialog.
 *
 * @since 2.0.0
 */
interface SaveDialogOptions {
  /** The title of the dialog window. */
  title?: string;
  /** The filters of the dialog. */
  filters?: DialogFilter[];
  /**
   * Initial directory or file path.
   * If it's a directory path, the dialog interface will change to that folder.
   * If it's not an existing directory, the file name will be set to the dialog's file name input and the dialog will be set to the parent folder.
   */
  defaultPath?: string;
}

/**
 * @since 2.0.0
 */
interface MessageDialogOptions {
  /** The title of the dialog. Defaults to the app name. */
  title?: string;
  /** The type of the dialog. Defaults to `info`. */
  type?: "info" | "warning" | "error";
  /** The label of the confirm button. */
  okLabel?: string;
}

interface ConfirmDialogOptions {
  /** The title of the dialog. Defaults to the app name. */
  title?: string;
  /** The type of the dialog. Defaults to `info`. */
  type?: "info" | "warning" | "error";
  /** The label of the confirm button. */
  okLabel?: string;
  /** The label of the cancel button. */
  cancelLabel?: string;
}

async function open(
  options?: OpenDialogOptions & { multiple?: false; directory?: false },
): Promise<null | FileResponse>;
async function open(
  options?: OpenDialogOptions & { multiple?: true; directory?: false },
): Promise<null | FileResponse[]>;
async function open(
  options?: OpenDialogOptions & { multiple?: false; directory?: true },
): Promise<null | string>;
async function open(
  options?: OpenDialogOptions & { multiple?: true; directory?: true },
): Promise<null | string[]>;
/**
 * Open a file/directory selection dialog.
 *
 * The selected paths are added to the filesystem and asset protocol scopes.
 * When security is more important than the easy of use of this API,
 * prefer writing a dedicated command instead.
 *
 * Note that the scope change is not persisted, so the values are cleared when the application is restarted.
 * You can save it to the filesystem using [tauri-plugin-persisted-scope](https://github.com/tauri-apps/tauri-plugin-persisted-scope).
 * @example
 * ```typescript
 * import { open } from '@tauri-apps/plugin-dialog';
 * // Open a selection dialog for image files
 * const selected = await open({
 *   multiple: true,
 *   filters: [{
 *     name: 'Image',
 *     extensions: ['png', 'jpeg']
 *   }]
 * });
 * if (Array.isArray(selected)) {
 *   // user selected multiple files
 * } else if (selected === null) {
 *   // user cancelled the selection
 * } else {
 *   // user selected a single file
 * }
 * ```
 *
 * @example
 * ```typescript
 * import { open } from '@tauri-apps/plugin-dialog';
 * import { appDir } from '@tauri-apps/api/path';
 * // Open a selection dialog for directories
 * const selected = await open({
 *   directory: true,
 *   multiple: true,
 *   defaultPath: await appDir(),
 * });
 * if (Array.isArray(selected)) {
 *   // user selected multiple directories
 * } else if (selected === null) {
 *   // user cancelled the selection
 * } else {
 *   // user selected a single directory
 * }
 * ```
 *
 * @returns A promise resolving to the selected path(s)
 *
 * @since 2.0.0
 */
async function open(
  options: OpenDialogOptions = {},
): Promise<null | string | string[] | FileResponse | FileResponse[]> {
  if (typeof options === "object") {
    Object.freeze(options);
  }

  return invoke("plugin:dialog|open", { options });
}

/**
 * Open a file/directory save dialog.
 *
 * The selected path is added to the filesystem and asset protocol scopes.
 * When security is more important than the easy of use of this API,
 * prefer writing a dedicated command instead.
 *
 * Note that the scope change is not persisted, so the values are cleared when the application is restarted.
 * You can save it to the filesystem using [tauri-plugin-persisted-scope](https://github.com/tauri-apps/tauri-plugin-persisted-scope).
 * @example
 * ```typescript
 * import { save } from '@tauri-apps/plugin-dialog';
 * const filePath = await save({
 *   filters: [{
 *     name: 'Image',
 *     extensions: ['png', 'jpeg']
 *   }]
 * });
 * ```
 *
 * @returns A promise resolving to the selected path.
 *
 * @since 2.0.0
 */
async function save(options: SaveDialogOptions = {}): Promise<string | null> {
  if (typeof options === "object") {
    Object.freeze(options);
  }

  return invoke("plugin:dialog|save", { options });
}

/**
 * Shows a message dialog with an `Ok` button.
 * @example
 * ```typescript
 * import { message } from '@tauri-apps/plugin-dialog';
 * await message('Tauri is awesome', 'Tauri');
 * await message('File not found', { title: 'Tauri', type: 'error' });
 * ```
 *
 * @param message The message to show.
 * @param options The dialog's options. If a string, it represents the dialog title.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 *
 */
async function message(
  message: string,
  options?: string | MessageDialogOptions,
): Promise<void> {
  const opts = typeof options === "string" ? { title: options } : options;
  return invoke("plugin:dialog|message", {
    message: message.toString(),
    title: opts?.title?.toString(),
    type_: opts?.type,
    okButtonLabel: opts?.okLabel?.toString(),
  });
}

/**
 * Shows a question dialog with `Yes` and `No` buttons.
 * @example
 * ```typescript
 * import { ask } from '@tauri-apps/plugin-dialog';
 * const yes = await ask('Are you sure?', 'Tauri');
 * const yes2 = await ask('This action cannot be reverted. Are you sure?', { title: 'Tauri', type: 'warning' });
 * ```
 *
 * @param message The message to show.
 * @param options The dialog's options. If a string, it represents the dialog title.
 *
 * @returns A promise resolving to a boolean indicating whether `Yes` was clicked or not.
 *
 * @since 2.0.0
 */
async function ask(
  message: string,
  options?: string | ConfirmDialogOptions,
): Promise<boolean> {
  const opts = typeof options === "string" ? { title: options } : options;
  return invoke("plugin:dialog|ask", {
    message: message.toString(),
    title: opts?.title?.toString(),
    type_: opts?.type,
    okButtonLabel: opts?.okLabel?.toString() ?? "Yes",
    cancelButtonLabel: opts?.cancelLabel?.toString() ?? "No",
  });
}

/**
 * Shows a question dialog with `Ok` and `Cancel` buttons.
 * @example
 * ```typescript
 * import { confirm } from '@tauri-apps/plugin-dialog';
 * const confirmed = await confirm('Are you sure?', 'Tauri');
 * const confirmed2 = await confirm('This action cannot be reverted. Are you sure?', { title: 'Tauri', type: 'warning' });
 * ```
 *
 * @param message The message to show.
 * @param options The dialog's options. If a string, it represents the dialog title.
 *
 * @returns A promise resolving to a boolean indicating whether `Ok` was clicked or not.
 *
 * @since 2.0.0
 */
async function confirm(
  message: string,
  options?: string | ConfirmDialogOptions,
): Promise<boolean> {
  const opts = typeof options === "string" ? { title: options } : options;
  return invoke("plugin:dialog|confirm", {
    message: message.toString(),
    title: opts?.title?.toString(),
    type_: opts?.type,
    okButtonLabel: opts?.okLabel?.toString() ?? "Ok",
    cancelButtonLabel: opts?.cancelLabel?.toString() ?? "Cancel",
  });
}

export type {
  DialogFilter,
  FileResponse,
  OpenDialogOptions,
  SaveDialogOptions,
  MessageDialogOptions,
  ConfirmDialogOptions,
};

export { open, save, message, ask, confirm };
