import { invoke } from '@tauri-apps/api/tauri';

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
async function metadata(path) {
    return await invoke("plugin:fs-extra|metadata", {
        path,
    }).then((metadata) => {
        const { accessedAtMs, createdAtMs, modifiedAtMs, ...data } = metadata;
        return {
            accessedAt: new Date(accessedAtMs),
            createdAt: new Date(createdAtMs),
            modifiedAt: new Date(modifiedAtMs),
            ...data,
        };
    });
}
async function exists(path) {
    return await invoke("plugin:fs-extra|exists", { path });
}

export { exists, metadata };
//# sourceMappingURL=index.mjs.map
