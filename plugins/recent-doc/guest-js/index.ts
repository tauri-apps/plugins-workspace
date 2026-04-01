// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

/**
 * Manage the operating system recent documents list.
 *
 * @module
 */

/**
 * Adds a document to this app's recent documents list.
 *
 * #### Requirements
 *
 * - `path` must be a local filesystem path.
 * - For installer-based Windows builds, define handled extensions in `tauri.conf.json` at `bundle.fileAssociations` so associations are created on install and removed on uninstall.
 *
 * #### Platform-specific
 *
 * - **Linux / Android / iOS:** Unsupported.
 *
 * @param path Local filesystem path to the document.
 *
 * @example
 * ```typescript
 * import { addRecentDocument } from '@tauri-apps/plugin-recent-doc';
 * await addRecentDocument('/path/to/document.txt');
 * ```
 *
 * This function rejects when called on an unsupported platform.
 * On Windows, it also rejects if the native shell API fails (for example, when the path cannot be resolved).
 *
 * @since 2.0.0
 */
async function addRecentDocument(path: string): Promise<void> {
  return invoke('plugin:recent-doc|add_recent_document', { path })
}

/**
 * Gets this app's recent documents list from the operating system.
 *
 * #### Platform-specific
 *
 * - **Linux / Android / iOS:** Unsupported.
 *
 * @example
 * ```typescript
 * import { getRecentDocuments } from '@tauri-apps/plugin-recent-doc';
 * const recent = await getRecentDocuments();
 * ```
 *
 * This function rejects when called on an unsupported platform.
 * On Windows, it also rejects if the native shell API fails.
 *
 * @since 2.0.0
 */
async function getRecentDocuments(): Promise<string[]> {
  return invoke<string[]>('plugin:recent-doc|get_recent_documents')
}

/**
 * Clears this app's recent documents list in the operating system.
 *
 * #### Platform-specific
 *
 * - **Linux / Android / iOS:** Unsupported.
 *
 * @example
 * ```typescript
 * import { clearRecentDocuments } from '@tauri-apps/plugin-recent-doc';
 * await clearRecentDocuments();
 * ```
 *
 * This function rejects when called on an unsupported platform.
 * On Windows, it also rejects if the native shell API fails.
 *
 * @since 2.0.0
 */
async function clearRecentDocuments(): Promise<void> {
  return invoke('plugin:recent-doc|clear_recent_documents')
}

export { addRecentDocument, getRecentDocuments, clearRecentDocuments }
