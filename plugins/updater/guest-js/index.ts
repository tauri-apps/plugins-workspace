// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke, Channel, Resource } from '@tauri-apps/api/core'

/** Options used when checking for updates */
interface CheckOptions {
  /**
   * Request headers
   */
  headers?: HeadersInit
  /**
   * Timeout in milliseconds
   */
  timeout?: number
  /**
   * A proxy url to be used when checking and downloading updates.
   */
  proxy?: string
  /**
   * Target identifier for the running application. This is sent to the backend.
   */
  target?: string
}

/** Options used when downloading an update */
interface DownloadOptions {
  /**
   * Request headers
   */
  headers?: HeadersInit
  /**
   * Timeout in milliseconds
   */
  timeout?: number
}

interface UpdateMetadata {
  rid: number
  available: boolean
  currentVersion: string
  version: string
  date?: string
  body?: string
  rawJson: Record<string, unknown>
}

/** Updater download event */
type DownloadEvent =
  | { event: 'Started'; data: { contentLength?: number } }
  | { event: 'Progress'; data: { chunkLength: number } }
  | { event: 'Finished' }

class Update extends Resource {
  available: boolean
  currentVersion: string
  version: string
  date?: string
  body?: string
  rawJson: Record<string, unknown>
  private downloadedBytes?: Resource

  constructor(metadata: UpdateMetadata) {
    super(metadata.rid)
    this.available = metadata.available
    this.currentVersion = metadata.currentVersion
    this.version = metadata.version
    this.date = metadata.date
    this.body = metadata.body
    this.rawJson = metadata.rawJson
  }

  /** Download the updater package */
  async download(
    onEvent?: (progress: DownloadEvent) => void,
    options?: DownloadOptions
  ): Promise<void> {
    const channel = new Channel<DownloadEvent>()
    if (onEvent) {
      channel.onmessage = onEvent
    }
    const downloadedBytesRid = await invoke<number>('plugin:updater|download', {
      onEvent: channel,
      rid: this.rid,
      ...options
    })
    this.downloadedBytes = new Resource(downloadedBytesRid)
  }

  /** Install downloaded updater package */
  async install(): Promise<void> {
    if (!this.downloadedBytes) {
      throw new Error('Update.install called before Update.download')
    }

    await invoke('plugin:updater|install', {
      updateRid: this.rid,
      bytesRid: this.downloadedBytes.rid
    })

    // Don't need to call close, we did it in rust side already
    this.downloadedBytes = undefined
  }

  /** Downloads the updater package and installs it */
  async downloadAndInstall(
    onEvent?: (progress: DownloadEvent) => void,
    options?: DownloadOptions
  ): Promise<void> {
    const channel = new Channel<DownloadEvent>()
    if (onEvent) {
      channel.onmessage = onEvent
    }
    await invoke('plugin:updater|download_and_install', {
      onEvent: channel,
      rid: this.rid,
      ...options
    })
  }

  async close(): Promise<void> {
    await this.downloadedBytes?.close()
    await super.close()
  }
}

/** Check for updates, resolves to `null` if no updates are available */
async function check(options?: CheckOptions): Promise<Update | null> {
  if (options?.headers) {
    options.headers = Array.from(new Headers(options.headers).entries())
  }

  return await invoke<UpdateMetadata>('plugin:updater|check', {
    ...options
  }).then((meta) => (meta.available ? new Update(meta) : null))
}

export type { CheckOptions, DownloadOptions, DownloadEvent }
export { check, Update }
