// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke, Channel } from "@tauri-apps/api/tauri";

interface CheckOptions {
  /**
   * Request headers
   */
  headers?: Record<string, unknown>;
  /**
   * Timeout in seconds
   */
  timeout?: number;
  /**
   * Target identifier for the running application. This is sent to the backend.
   */
  target?: string;
}

interface UpdateResponse {
  available: boolean;
  currentVersion: string;
  latestVersion: string;
  date?: string;
  body?: string;
}

type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

class Update {
  response: UpdateResponse;

  constructor(response: UpdateResponse) {
    this.response = response;
  }

  async downloadAndInstall(
    onEvent?: (progress: DownloadEvent) => void
  ): Promise<void> {
    const channel = new Channel<DownloadEvent>();
    if (onEvent != null) {
      channel.onmessage = onEvent;
    }
    return invoke("plugin:updater|download_and_install", { onEvent: channel });
  }
}

async function check(options?: CheckOptions): Promise<Update> {
  return invoke<UpdateResponse>("plugin:updater|check", { ...options }).then(
    (response) => new Update(response)
  );
}

export type { CheckOptions, UpdateResponse, DownloadEvent };
export { check, Update };
