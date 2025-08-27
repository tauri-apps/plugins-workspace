// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

/**
 * Extension filters for the file dialog.
 *
 * @since 2.0.0
 */
interface DialogFilter {
  /** Filter name. */
  name: string
  /**
   * Extensions to filter, without a `.` prefix.
   * @example
   * ```typescript
   * extensions: ['svg', 'png']
   * ```
   */
  extensions: string[]
}

/**
 * Options for the open dialog.
 *
 * @since 2.0.0
 */
interface OpenDialogOptions {
  /** The title of the dialog window (desktop only). */
  title?: string
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /**
   * Initial directory or file path.
   * If it's a directory path, the dialog interface will change to that folder.
   * If it's not an existing directory, the file name will be set to the dialog's file name input and the dialog will be set to the parent folder.
   *
   * On mobile the file name is always used on the dialog's file name input.
   * If not provided, Android uses `(invalid).txt` as default file name.
   */
  defaultPath?: string
  /** Whether the dialog allows multiple selection or not. */
  multiple?: boolean
  /** Whether the dialog is a directory selection or not. */
  directory?: boolean
  /**
   * If `directory` is true, indicates that it will be read recursively later.
   * Defines whether subdirectories will be allowed on the scope or not.
   */
  recursive?: boolean
  /** Whether to allow creating directories in the dialog. Enabled by default. **macOS Only** */
  canCreateDirectories?: boolean
}

/**
 * Options for the save dialog.
 *
 * @since 2.0.0
 */
interface SaveDialogOptions {
  /** The title of the dialog window (desktop only). */
  title?: string
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /**
   * Initial directory or file path.
   * If it's a directory path, the dialog interface will change to that folder.
   * If it's not an existing directory, the file name will be set to the dialog's file name input and the dialog will be set to the parent folder.
   *
   * On mobile the file name is always used on the dialog's file name input.
   * If not provided, Android uses `(invalid).txt` as default file name.
   */
  defaultPath?: string
  /** Whether to allow creating directories in the dialog. Enabled by default. **macOS Only** */
  canCreateDirectories?: boolean
}

/**
 * Default buttons for a message dialog.
 *
 * @since 2.4.0
 */
export type MessageDialogDefaultButtons =
  | 'Ok'
  | 'OkCancel'
  | 'YesNo'
  | 'YesNoCancel'

/** All possible button keys. */
type ButtonKey = 'ok' | 'cancel' | 'yes' | 'no'

/** Ban everything except a set of keys. */
type BanExcept<Allowed extends ButtonKey> = Partial<
  Record<Exclude<ButtonKey, Allowed>, never>
>

/**
 * The Yes, No and Cancel buttons of a message dialog.
 *
 * @since 2.4.0
 */
export type MessageDialogButtonsYesNoCancel = {
  /** The Yes button. */
  yes: string
  /** The No button. */
  no: string
  /** The Cancel button. */
  cancel: string
} & BanExcept<'yes' | 'no' | 'cancel'>

/**
 * The Ok and Cancel buttons of a message dialog.
 *
 * @since 2.4.0
 */
export type MessageDialogButtonsOkCancel = {
  /** The Ok button. */
  ok: string
  /** The Cancel button. */
  cancel: string
} & BanExcept<'ok' | 'cancel'>

/**
 * The Ok button of a message dialog.
 *
 * @since 2.4.0
 */
export type MessageDialogButtonsOk = {
  /** The Ok button. */
  ok: string
} & BanExcept<'ok'>

/**
 * Custom buttons for a message dialog.
 *
 * @since 2.4.0
 */
export type MessageDialogCustomButtons =
  | MessageDialogButtonsYesNoCancel
  | MessageDialogButtonsOkCancel
  | MessageDialogButtonsOk

/**
 * The buttons of a message dialog.
 *
 * @since 2.4.0
 */
export type MessageDialogButtons =
  | MessageDialogDefaultButtons
  | MessageDialogCustomButtons

/**
 * @since 2.0.0
 */
interface MessageDialogOptions {
  /** The title of the dialog. Defaults to the app name. */
  title?: string
  /** The kind of the dialog. Defaults to `info`. */
  kind?: 'info' | 'warning' | 'error'
  /**
   * The label of the Ok button.
   *
   * @deprecated Use {@linkcode MessageDialogOptions.buttons} instead.
   */
  okLabel?: string
  /**
   * The buttons of the dialog.
   *
   * @example
   *
   * ```ts
   * // Use system default buttons texts
   * await message('Hello World!', { buttons: 'Ok' })
   * await message('Hello World!', { buttons: 'OkCancel' })
   *
   * // Or with custom button texts
   * await message('Hello World!', { buttons: { ok: 'Yes!' } })
   * await message('Take on the task?', {
   *   buttons: { ok: 'Accept', cancel: 'Cancel' }
   * })
   * await message('Show the file content?', {
   *   buttons: { yes: 'Show content', no: 'Show in folder', cancel: 'Cancel' }
   * })
   * ```
   *
   * @since 2.4.0
   */
  buttons?: MessageDialogButtons
}

/**
 * Internal function to convert the buttons to the Rust type.
 */
function buttonsToRust(buttons: MessageDialogButtons | undefined) {
  if (buttons === undefined) {
    return undefined
  }

  if (typeof buttons === 'string') {
    return buttons
  } else if ('ok' in buttons && 'cancel' in buttons) {
    return { OkCancelCustom: [buttons.ok, buttons.cancel] }
  } else if ('yes' in buttons && 'no' in buttons && 'cancel' in buttons) {
    return {
      YesNoCancelCustom: [buttons.yes, buttons.no, buttons.cancel]
    }
  } else if ('ok' in buttons) {
    return { OkCustom: buttons.ok }
  }

  return undefined
}

interface ConfirmDialogOptions {
  /** The title of the dialog. Defaults to the app name. */
  title?: string
  /** The kind of the dialog. Defaults to `info`. */
  kind?: 'info' | 'warning' | 'error'
  /** The label of the confirm button. */
  okLabel?: string
  /** The label of the cancel button. */
  cancelLabel?: string
}

type OpenDialogReturn<T extends OpenDialogOptions> = T['directory'] extends true
  ? T['multiple'] extends true
    ? string[] | null
    : string | null
  : T['multiple'] extends true
    ? string[] | null
    : string | null

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
async function open<T extends OpenDialogOptions>(
  options: T = {} as T
): Promise<OpenDialogReturn<T>> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return await invoke('plugin:dialog|open', { options })
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
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return await invoke('plugin:dialog|save', { options })
}

/**
 * The result of a message dialog.
 *
 * The result is a string if the dialog has custom buttons,
 * otherwise it is one of the default buttons.
 *
 * @since 2.4.0
 */
export type MessageDialogResult = 'Yes' | 'No' | 'Ok' | 'Cancel' | (string & {})

/**
 * Shows a message dialog with an `Ok` button.
 * @example
 * ```typescript
 * import { message } from '@tauri-apps/plugin-dialog';
 * await message('Tauri is awesome', 'Tauri');
 * await message('File not found', { title: 'Tauri', kind: 'error' });
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
  options?: string | MessageDialogOptions
): Promise<MessageDialogResult> {
  const opts = typeof options === 'string' ? { title: options } : options

  return invoke<MessageDialogResult>('plugin:dialog|message', {
    message: message.toString(),
    title: opts?.title?.toString(),
    kind: opts?.kind,
    okButtonLabel: opts?.okLabel?.toString(),
    buttons: buttonsToRust(opts?.buttons)
  })
}

/**
 * Shows a question dialog with `Yes` and `No` buttons.
 * @example
 * ```typescript
 * import { ask } from '@tauri-apps/plugin-dialog';
 * const yes = await ask('Are you sure?', 'Tauri');
 * const yes2 = await ask('This action cannot be reverted. Are you sure?', { title: 'Tauri', kind: 'warning' });
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
  options?: string | ConfirmDialogOptions
): Promise<boolean> {
  const opts = typeof options === 'string' ? { title: options } : options
  return await invoke('plugin:dialog|ask', {
    message: message.toString(),
    title: opts?.title?.toString(),
    kind: opts?.kind,
    yesButtonLabel: opts?.okLabel?.toString(),
    noButtonLabel: opts?.cancelLabel?.toString()
  })
}

/**
 * Shows a question dialog with `Ok` and `Cancel` buttons.
 * @example
 * ```typescript
 * import { confirm } from '@tauri-apps/plugin-dialog';
 * const confirmed = await confirm('Are you sure?', 'Tauri');
 * const confirmed2 = await confirm('This action cannot be reverted. Are you sure?', { title: 'Tauri', kind: 'warning' });
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
  options?: string | ConfirmDialogOptions
): Promise<boolean> {
  const opts = typeof options === 'string' ? { title: options } : options
  return await invoke('plugin:dialog|confirm', {
    message: message.toString(),
    title: opts?.title?.toString(),
    kind: opts?.kind,
    okButtonLabel: opts?.okLabel?.toString(),
    cancelButtonLabel: opts?.cancelLabel?.toString()
  })
}

export type {
  DialogFilter,
  OpenDialogOptions,
  OpenDialogReturn,
  SaveDialogOptions,
  MessageDialogOptions,
  ConfirmDialogOptions
}

export { open, save, message, ask, confirm }
