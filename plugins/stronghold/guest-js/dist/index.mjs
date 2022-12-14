import { invoke } from '@tauri-apps/api/tauri';

function toBytesDto(v) {
    if (typeof v === "string") {
        return v;
    }
    return Array.from(v instanceof ArrayBuffer ? new Uint8Array(v) : v);
}
class Location {
    constructor(type, payload) {
        this.type = type;
        this.payload = payload;
    }
    static generic(vault, record) {
        return new Location("Generic", {
            vault: toBytesDto(vault),
            record: toBytesDto(record),
        });
    }
    static counter(vault, counter) {
        return new Location("Counter", {
            vault: toBytesDto(vault),
            counter,
        });
    }
}
class ProcedureExecutor {
    constructor(procedureArgs) {
        this.procedureArgs = procedureArgs;
    }
    async generateSLIP10Seed(outputLocation, sizeBytes) {
        return await invoke("plugin:stronghold|execute_procedure", {
            ...this.procedureArgs,
            procedure: {
                type: "SLIP10Generate",
                payload: {
                    output: outputLocation,
                    sizeBytes,
                },
            },
        }).then((n) => Uint8Array.from(n));
    }
    async deriveSLIP10(chain, source, sourceLocation, outputLocation) {
        return await invoke("plugin:stronghold|execute_procedure", {
            ...this.procedureArgs,
            procedure: {
                type: "SLIP10Derive",
                payload: {
                    chain,
                    input: {
                        type: source,
                        payload: sourceLocation,
                    },
                    output: outputLocation,
                },
            },
        }).then((n) => Uint8Array.from(n));
    }
    async recoverBIP39(mnemonic, outputLocation, passphrase) {
        return await invoke("plugin:stronghold|execute_procedure", {
            ...this.procedureArgs,
            procedure: {
                type: "BIP39Recover",
                payload: {
                    mnemonic,
                    passphrase,
                    output: outputLocation,
                },
            },
        }).then((n) => Uint8Array.from(n));
    }
    async generateBIP39(outputLocation, passphrase) {
        return await invoke("plugin:stronghold|execute_procedure", {
            ...this.procedureArgs,
            procedure: {
                type: "BIP39Generate",
                payload: {
                    output: outputLocation,
                    passphrase,
                },
            },
        }).then((n) => Uint8Array.from(n));
    }
    async getEd25519PublicKey(privateKeyLocation) {
        return await invoke("plugin:stronghold|execute_procedure", {
            ...this.procedureArgs,
            procedure: {
                type: "PublicKey",
                payload: {
                    type: "Ed25519",
                    privateKey: privateKeyLocation,
                },
            },
        }).then((n) => Uint8Array.from(n));
    }
    async signEd25519(privateKeyLocation, msg) {
        return await invoke("plugin:stronghold|execute_procedure", {
            ...this.procedureArgs,
            procedure: {
                type: "Ed25519Sign",
                payload: {
                    privateKey: privateKeyLocation,
                    msg,
                },
            },
        }).then((n) => Uint8Array.from(n));
    }
}
class Client {
    constructor(path, name) {
        this.path = path;
        this.name = toBytesDto(name);
    }
    getVault(name) {
        return new Vault(this.path, this.name, toBytesDto(name));
    }
    getStore() {
        return new Store(this.path, this.name);
    }
}
class Store {
    constructor(path, client) {
        this.path = path;
        this.client = client;
    }
    async get(key) {
        return await invoke("plugin:stronghold|get_store_record", {
            snapshotPath: this.path,
            client: this.client,
            key: toBytesDto(key),
        }).then((v) => Uint8Array.from(v));
    }
    async insert(key, value, lifetime) {
        return await invoke("plugin:stronghold|save_store_record", {
            snapshotPath: this.path,
            client: this.client,
            key: toBytesDto(key),
            value,
            lifetime,
        });
    }
    async remove(key) {
        return await invoke("plugin:stronghold|remove_store_record", {
            snapshotPath: this.path,
            client: this.client,
            key: toBytesDto(key),
        }).then((v) => (v != null ? Uint8Array.from(v) : null));
    }
}
class Vault extends ProcedureExecutor {
    constructor(path, client, name) {
        super({
            snapshotPath: path,
            client,
            vault: name,
        });
        this.path = path;
        this.client = toBytesDto(client);
        this.name = toBytesDto(name);
    }
    async insert(recordPath, secret) {
        return await invoke("plugin:stronghold|save_secret", {
            snapshotPath: this.path,
            client: this.client,
            vault: this.name,
            recordPath: toBytesDto(recordPath),
            secret,
        });
    }
    async remove(location) {
        return await invoke("plugin:stronghold|remove_secret", {
            snapshotPath: this.path,
            client: this.client,
            vault: this.name,
            location,
        });
    }
}
class Stronghold {
    constructor(path, password) {
        this.path = path;
        void this.reload(password);
    }
    async reload(password) {
        return await invoke("plugin:stronghold|initialize", {
            snapshotPath: this.path,
            password,
        });
    }
    async unload() {
        return await invoke("plugin:stronghold|destroy", {
            snapshotPath: this.path,
        });
    }
    async loadClient(client) {
        return await invoke("plugin:stronghold|load_client", {
            snapshotPath: this.path,
            client: toBytesDto(client),
        }).then(() => new Client(this.path, client));
    }
    async createClient(client) {
        return await invoke("plugin:stronghold|create_client", {
            snapshotPath: this.path,
            client: toBytesDto(client),
        }).then(() => new Client(this.path, client));
    }
    async save() {
        return await invoke("plugin:stronghold|save", {
            snapshotPath: this.path,
        });
    }
}

export { Client, Location, Store, Stronghold, Vault };
//# sourceMappingURL=index.mjs.map
