import { invoke } from '@tauri-apps/api/tauri';
import { appWindow } from '@tauri-apps/api/window';

const handlers = new Map();
let listening = false;
async function listenToUploadEventIfNeeded() {
    if (listening) {
        return await Promise.resolve();
    }
    return await appWindow
        .listen("upload://progress", ({ payload }) => {
        const handler = handlers.get(payload.id);
        if (handler != null) {
            handler(payload.progress, payload.total);
        }
    })
        .then(() => {
        listening = true;
    });
}
async function upload(url, filePath, progressHandler, headers) {
    const ids = new Uint32Array(1);
    window.crypto.getRandomValues(ids);
    const id = ids[0];
    if (progressHandler != null) {
        handlers.set(id, progressHandler);
    }
    await listenToUploadEventIfNeeded();
    await invoke("plugin:upload|upload", {
        id,
        url,
        filePath,
        headers: headers !== null && headers !== void 0 ? headers : {},
    });
}

export { upload as default };
//# sourceMappingURL=index.mjs.map
