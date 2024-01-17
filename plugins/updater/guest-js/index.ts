// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke, Channel } from "@tauri-apps/api/core";

/** Options used to check for updates */
interface CheckOptions {
  /**
   * Request headers
   */
  headers?: HeadersInit;
  /**
   * Timeout in seconds
   */
  timeout?: number;
  /**
   * A proxy url to be used when checking and downloading updates.
   */
  proxy?: string;
  /**
   * Target identifier for the running application. This is sent to the backend.
   */
  target?: string;
}

interface UpdateMetadata {
  available: boolean;
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
}

/** Updater download event */
type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

class Update {
  available: boolean;
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;

  constructor(metadata: UpdateMetadata) {
    this.available = metadata.available;
    this.currentVersion = metadata.currentVersion;
    this.version = metadata.version;
    this.date = metadata.date;
    this.body = metadata.body;
  }

  /** Downloads the updater package and installs it */
  async downloadAndInstall(
    onEvent?: (progress: DownloadEvent) => void,
  ): Promise<void> {
    const channel = new Channel<DownloadEvent>();
    if (onEvent != null) {
      channel.onmessage = onEvent;
    }
    return invoke("plugin:updater|download_and_install", {
      onEvent: channel,
    });
  }
}

/** Check for updates, resolves to `null` if no updates are available */
async function check(options?: CheckOptions): Promise<Update | null> {
  if (options?.headers) {
    options.headers = Array.from(new Headers(options.headers).entries());
  }

  return invoke<UpdateMetadata>("plugin:updater|check", { ...options }).then(
    (meta) => (meta.available ? new Update(meta) : null),
  );
}

export type { CheckOptions, DownloadEvent };
export { check, Update };
