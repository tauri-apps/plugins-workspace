// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke, Channel } from '@tauri-apps/api/core'

interface ProgressPayload {
  progress: number
  progressTotal: number
  total: number
  transferSpeed: number
}

type ProgressHandler = (progress: ProgressPayload) => void

async function upload(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  headers?: Map<string, string>
): Promise<string> {
  const ids = new Uint32Array(1)
  window.crypto.getRandomValues(ids)
  const id = ids[0]

  const onProgress = new Channel<ProgressPayload>()
  if (progressHandler) {
    onProgress.onmessage = progressHandler
  }

  return await invoke('plugin:upload|upload', {
    id,
    url,
    filePath,
    headers: headers ?? {},
    onProgress
  })
}

/// Download file from given url.
///
/// Note that `filePath` currently must include the file name.
/// Furthermore the progress events will report a total length of 0 if the server did not sent a `Content-Length` header or if the file is compressed.
async function download(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  headers?: Map<string, string>,
  body?: string
): Promise<void> {
  const ids = new Uint32Array(1)
  window.crypto.getRandomValues(ids)
  const id = ids[0]

  const onProgress = new Channel<ProgressPayload>()
  if (progressHandler) {
    onProgress.onmessage = progressHandler
  }

  await invoke('plugin:upload|download', {
    id,
    url,
    filePath,
    headers: headers ?? {},
    onProgress,
    body
  })
}

export { download, upload }
