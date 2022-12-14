import { invoke } from '@tauri-apps/api/tauri';
import { appWindow } from '@tauri-apps/api/window';

const w = appWindow;
async function unwatch(id) {
    await invoke("plugin:fs-watch|unwatch", { id });
}
async function watch(paths, options, cb) {
    const opts = {
        recursive: false,
        delayMs: 2000,
        ...options,
    };
    let watchPaths;
    if (typeof paths === "string") {
        watchPaths = [paths];
    }
    else {
        watchPaths = paths;
    }
    const id = window.crypto.getRandomValues(new Uint32Array(1))[0];
    await invoke("plugin:fs-watch|watch", {
        id,
        paths: watchPaths,
        options: opts,
    });
    const unlisten = await w.listen(`watcher://debounced-event/${id}`, (event) => {
        cb(event.payload);
    });
    return () => {
        void unwatch(id);
        unlisten();
    };
}
async function watchImmediate(paths, options, cb) {
    const opts = {
        recursive: false,
        ...options,
        delayMs: null,
    };
    let watchPaths;
    if (typeof paths === "string") {
        watchPaths = [paths];
    }
    else {
        watchPaths = paths;
    }
    const id = window.crypto.getRandomValues(new Uint32Array(1))[0];
    await invoke("plugin:fs-watch|watch", {
        id,
        paths: watchPaths,
        options: opts,
    });
    const unlisten = await w.listen(`watcher://raw-event/${id}`, (event) => {
        cb(event.payload);
    });
    return () => {
        void unwatch(id);
        unlisten();
    };
}

export { watch, watchImmediate };
//# sourceMappingURL=index.mjs.map
