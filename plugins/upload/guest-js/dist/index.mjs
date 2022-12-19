import { invoke } from '@tauri-apps/api/tauri';
import { appWindow } from '@tauri-apps/api/window';

var UploadEvent;
(function (UploadEvent) {
    UploadEvent["progress"] = "upload://progress";
    UploadEvent["fileSize"] = "upload://file-size";
})(UploadEvent || (UploadEvent = {}));
const handlers = new Map();
const listeningMap = new Map();
const getIdForEvent = (event, id) => `${event}-${id}`;
async function listenToEventIfNeeded(event) {
    if (listeningMap.get(event)) {
        return;
    }
    return appWindow.listen(event, ({ payload }) => {
        const eventId = getIdForEvent(event, payload.id);
        const handler = handlers.get(eventId);
        if (typeof handler === 'function') {
            handler(payload);
        }
    });
}
async function upload(url, filePath, progressHandler, fileSizeHandler, headers) {
    const ids = new Uint32Array(1);
    window.crypto.getRandomValues(ids);
    const id = ids[0];
    if (progressHandler) {
        const eventId = getIdForEvent(UploadEvent.progress, id);
        handlers.set(eventId, progressHandler);
    }
    if (fileSizeHandler) {
        const eventId = getIdForEvent(UploadEvent.fileSize, id);
        handlers.set(eventId, fileSizeHandler);
    }
    await listenToEventIfNeeded(UploadEvent.progress);
    await listenToEventIfNeeded(UploadEvent.fileSize);
    return await invoke('plugin:upload|upload', {
        id,
        url,
        filePath,
        headers: headers !== null && headers !== void 0 ? headers : {},
    });
}

export { upload as default };
//# sourceMappingURL=index.mjs.map
