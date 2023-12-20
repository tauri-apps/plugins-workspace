import { invoke } from "@tauri-apps/api/tauri";
import { UnlistenFn } from "@tauri-apps/api/event";
import { appWindow, WebviewWindow } from "@tauri-apps/api/window";

const w: WebviewWindow = appWindow;

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
  | { kind: "Any"; path: string }[]
  | { kind: "AnyContinuous"; path: string }[];

async function unwatch(id: number): Promise<void> {
  await invoke("plugin:fs-watch|unwatch", { id });
}

export async function watch(
  paths: string | string[],
  cb: (event: DebouncedEvent) => void,
  options: DebouncedWatchOptions = {},
): Promise<UnlistenFn> {
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

  await invoke("plugin:fs-watch|watch", {
    id,
    paths: watchPaths,
    options: opts,
  });

  const unlisten = await w.listen<DebouncedEvent>(
    `watcher://debounced-event/${id}`,
    (event) => {
      cb(event.payload);
    },
  );

  return () => {
    void unwatch(id);
    unlisten();
  };
}

export async function watchImmediate(
  paths: string | string[],
  cb: (event: RawEvent) => void,
  options: WatchOptions = {},
): Promise<UnlistenFn> {
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

  await invoke("plugin:fs-watch|watch", {
    id,
    paths: watchPaths,
    options: opts,
  });

  const unlisten = await w.listen<RawEvent>(
    `watcher://raw-event/${id}`,
    (event) => {
      cb(event.payload);
    },
  );

  return () => {
    void unwatch(id);
    unlisten();
  };
}
