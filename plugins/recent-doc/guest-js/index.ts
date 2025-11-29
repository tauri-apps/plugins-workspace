// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

async function addRecentDocument(path: string): Promise<void> {
  return invoke('plugin:recent-doc|add_recent_document', { path });
}

async function getRecentDocuments(): Promise<string[]> {
  return invoke<string[]>('plugin:recent-doc|get_recent_documents');
}

async function clearRecentDocuments(): Promise<void> {
  return invoke('plugin:recent-doc|clear_recent_documents');
}

export {
  addRecentDocument,
  getRecentDocuments,
  clearRecentDocuments
}
