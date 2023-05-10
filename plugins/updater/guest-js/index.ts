import { invoke, transformCallback } from '@tauri-apps/api/tauri'

interface CheckOptions {
  /**
   * Request headers
   */
  headers?: Record<string, unknown>
  /**
   * Timeout in seconds
   */
  timeout?: number
  /**
   * Target identifier for the running application. This is sent to the backend.
   */
  target?: string
}

interface UpdateResponse {
  available: boolean
  currentVersion: string
  latestVersion: string
  date?: string
  body?: string
}

// TODO: use channel from @tauri-apps/api on v2
class Channel<T = unknown> {
  id: number
  onmessage: (response: T) => void = () => {
    // do nothing
  }

  constructor() {
    this.id = transformCallback((response: T) => {
      this.onmessage(response)
    })
  }

  toJSON(): string {
    return `__CHANNEL__:${this.id}`
  }
}

type DownloadEvent =
  { event: 'Started', data: { contentLength?: number } } |
  { event: 'Progress', data: { chunkLength: number } } |
  { event: 'Finished' }

class Update {
  response: UpdateResponse

  private constructor(response: UpdateResponse) {
    this.response = response
  }

  async downloadAndInstall(onEvent?: (progress: DownloadEvent) => void): Promise<void> {
    const channel = new Channel<DownloadEvent>()
    if (onEvent != null) {
      channel.onmessage = onEvent
    }
    return invoke('plugin:updater|download_and_install', { onEvent: channel })
  }
}

async function check(options?: CheckOptions): Promise<Update> {
  return invoke('plugin:updater|check', { ...options })
}

export type { CheckOptions, UpdateResponse, DownloadEvent }
export { check, Update }
