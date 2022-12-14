import { invoke, transformCallback } from '@tauri-apps/api/tauri';

class WebSocket {
    constructor(id, listeners) {
        this.id = id;
        this.listeners = listeners;
    }
    static async connect(url, options) {
        const listeners = [];
        const handler = (message) => {
            listeners.forEach((l) => l(message));
        };
        return await invoke("plugin:websocket|connect", {
            url,
            callbackFunction: transformCallback(handler),
            options,
        }).then((id) => new WebSocket(id, listeners));
    }
    addListener(cb) {
        this.listeners.push(cb);
    }
    async send(message) {
        let m;
        if (typeof message === "string") {
            m = { type: "Text", data: message };
        }
        else if (typeof message === "object" && "type" in message) {
            m = message;
        }
        else if (Array.isArray(message)) {
            m = { type: "Binary", data: message };
        }
        else {
            throw new Error("invalid `message` type, expected a `{ type: string, data: any }` object, a string or a numeric array");
        }
        return await invoke("plugin:websocket|send", {
            id: this.id,
            message: m,
        });
    }
    async disconnect() {
        return await this.send({
            type: "Close",
            data: {
                code: 1000,
                reason: "Disconnected by client",
            },
        });
    }
}

export { WebSocket as default };
//# sourceMappingURL=index.mjs.map
