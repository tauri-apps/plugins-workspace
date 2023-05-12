import { invoke, transformCallback } from "@tauri-apps/api/tauri";

export interface WatchOptions {
  recursive?: boolean;
}

export interface DebouncedWatchOptions extends WatchOptions {
  delayMs?: number;
}

export type RawEvent = {
  type: RawEventKind;
  paths: string[];
  attrs: unknown;
};

type RawEventKind =
  | "any "
  | {
      access?: unknown;
    }
  | {
      create?: unknown;
    }
  | {
      modify?: unknown;
    }
  | {
      remove?: unknown;
    }
  | "other";

export type DebouncedEvent =
  | { kind: "any"; path: string }
  | { kind: "AnyContinous"; path: string };

async function unwatch(id: number): Promise<void> {
  await invoke("plugin:fs-watch|unwatch", { id });
}

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

export async function watch(
  paths: string | string[],
  cb: (event: DebouncedEvent) => void,
  options: DebouncedWatchOptions = {}
): Promise<() => void> {
  const opts = {
    recursive: false,
    delayMs: 2000,
    ...options,
  };
  let watchPaths;
  if (typeof paths === "string") {
    watchPaths = [paths];
  } else {
    watchPaths = paths;
  }

  const id = window.crypto.getRandomValues(new Uint32Array(1))[0];

  const onEvent = new Channel<DebouncedEvent>();
  onEvent.onmessage = cb;

  await invoke("plugin:fs-watch|watch", {
    id,
    paths: watchPaths,
    options: opts,
    onEvent,
  });

  return () => {
    void unwatch(id);
  };
}

export async function watchImmediate(
  paths: string | string[],
  cb: (event: RawEvent) => void,
  options: WatchOptions = {}
): Promise<() => void> {
  const opts = {
    recursive: false,
    ...options,
    delayMs: null,
  };
  let watchPaths;
  if (typeof paths === "string") {
    watchPaths = [paths];
  } else {
    watchPaths = paths;
  }

  const id = window.crypto.getRandomValues(new Uint32Array(1))[0];

  const onEvent = new Channel<RawEvent>();
  onEvent.onmessage = cb;

  await invoke("plugin:fs-watch|watch", {
    id,
    paths: watchPaths,
    options: opts,
    onEvent,
  });

  return () => {
    void unwatch(id);
  };
}
