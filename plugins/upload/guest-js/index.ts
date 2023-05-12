import { invoke, transformCallback } from "@tauri-apps/api/tauri";

interface ProgressPayload {
  progress: number;
  total: number;
}

type ProgressHandler = (progress: ProgressPayload) => void;

// TODO: use channel from @tauri-apps/api on v2
class Channel<T = unknown> {
  id: number;
  // @ts-expect-error field used by the IPC serializer
  private readonly __TAURI_CHANNEL_MARKER__ = true;
  #onmessage: (response: T) => void = () => {
    // no-op
  };

  constructor() {
    this.id = transformCallback((response: T) => {
      this.#onmessage(response);
    });
  }

  set onmessage(handler: (response: T) => void) {
    this.#onmessage = handler;
  }

  get onmessage(): (response: T) => void {
    return this.#onmessage;
  }

  toJSON(): string {
    return `__CHANNEL__:${this.id}`;
  }
}

async function upload(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  headers?: Map<string, string>
): Promise<void> {
  const ids = new Uint32Array(1);
  window.crypto.getRandomValues(ids);
  const id = ids[0];

  const onProgress = new Channel<ProgressPayload>();
  if (progressHandler != null) {
    onProgress.onmessage = progressHandler;
  }

  await invoke("plugin:upload|upload", {
    id,
    url,
    filePath,
    headers: headers ?? {},
    onProgress,
  });
}

/// Download file from given url.
///
/// Note that `filePath` currently must include the file name.
/// Furthermore the progress events will report a total length of 0 if the server did not sent a `Content-Length` header or if the file is compressed.
async function download(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  headers?: Map<string, string>
): Promise<void> {
  const ids = new Uint32Array(1);
  window.crypto.getRandomValues(ids);
  const id = ids[0];

  const onProgress = new Channel<ProgressPayload>();
  if (progressHandler != null) {
    onProgress.onmessage = progressHandler;
  }

  await invoke("plugin:upload|download", {
    id,
    url,
    filePath,
    headers: headers ?? {},
    onProgress,
  });
}

export default upload;
export { download, upload };
