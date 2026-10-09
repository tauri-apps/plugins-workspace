// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Upload files from disk to a remote server over HTTP, and download files from a remote HTTP
 * server to disk.
 *
 * @module
 */

import { invoke, Channel } from '@tauri-apps/api/core'

/**
 * The payload sent to a {@link ProgressHandler} while an upload or download is in progress.
 */
interface ProgressPayload {
  /**
   * The number of bytes transferred since the previous progress event (i.e. the size of the
   * last chunk), not the cumulative total.
   */
  progress: number
  /**
   * The cumulative number of bytes transferred so far.
   */
  progressTotal: number
  /**
   * The total size of the transfer in bytes. For a download this is the value of the response's
   * `Content-Length` header, and is `0` if the server did not send it or the response body is
   * compressed. For an upload this is the size of the file being uploaded.
   */
  total: number
  /**
   * The current transfer speed, approximately in bytes per second. It is recalculated about
   * every 500 milliseconds and stays at `0` until then.
   */
  transferSpeed: number
}

/**
 * A callback invoked with a {@link ProgressPayload} every time a chunk of data is
 * uploaded or downloaded.
 */
type ProgressHandler = (progress: ProgressPayload) => void

/**
 * The HTTP method used to send the file to the server in {@link upload}.
 */
enum HttpMethod {
  /**
   * Send the file using an HTTP `POST` request. This is the default when no method is given.
   */
  Post = 'POST',
  /**
   * Send the file using an HTTP `PUT` request.
   */
  Put = 'PUT',
  /**
   * Send the file using an HTTP `PATCH` request.
   */
  Patch = 'PATCH'
}

/**
 * Options for {@linkcode upload}.
 *
 * @since 3.0.0
 */
interface UploadOptions {
  /** Headers to send with the request. */
  headers?: Map<string, string> | Record<string, string>
  /** HTTP method to use. Defaults to {@linkcode HttpMethod.Post}. */
  method?: HttpMethod
}

/**
 * Options for {@linkcode download}.
 *
 * @since 3.0.0
 */
interface DownloadOptions {
  /** Headers to send with the request. */
  headers?: Map<string, string> | Record<string, string>
  /** Body to send with the request. When set, a `POST` request is made instead of a `GET`. */
  body?: string
}

function headersToRust(
  headers: Map<string, string> | Record<string, string> | undefined
): Record<string, string> {
  return headers instanceof Map ? Object.fromEntries(headers) : (headers ?? {})
}

/**
 * Upload a file to the given url.
 *
 * @example
 * ```typescript
 * import { upload, HttpMethod } from '@tauri-apps/plugin-upload';
 * const response = await upload(
 *   'https://example.com/file-upload',
 *   './path/to/my/file.txt',
 *   ({ progressTotal, total }) => console.log(`Uploaded ${progressTotal} of ${total} bytes`),
 *   { headers: { 'Content-Type': 'text/plain' }, method: HttpMethod.Put }
 * );
 * ```
 *
 * @returns The response body.
 */
async function upload(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  options?: UploadOptions
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
    headers: headersToRust(options?.headers),
    method: options?.method ?? HttpMethod.Post,
    onProgress
  })
}

/**
 * Download a file from the given url.
 *
 * Note that `filePath` currently must include the file name.
 * Furthermore the progress events will report a total length of 0 if the server did not sent a `Content-Length` header or if the file is compressed.
 *
 * @example
 * ```typescript
 * import { download } from '@tauri-apps/plugin-upload';
 * await download(
 *   'https://example.com/file-download-link',
 *   './path/to/save/my/file.txt',
 *   ({ progressTotal, total }) => console.log(`Downloaded ${progressTotal} of ${total} bytes`),
 *   { headers: { 'Content-Type': 'text/plain' } }
 * );
 * ```
 */
async function download(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  options?: DownloadOptions
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
    headers: headersToRust(options?.headers),
    onProgress,
    body: options?.body
  })
}

export { download, upload, HttpMethod }
export type { ProgressPayload, ProgressHandler, UploadOptions, DownloadOptions }
