type BytesDto = string | number[];
export type ClientPath = string | Iterable<number> | ArrayLike<number> | ArrayBuffer;
export type VaultPath = string | Iterable<number> | ArrayLike<number> | ArrayBuffer;
export type RecordPath = string | Iterable<number> | ArrayLike<number> | ArrayBuffer;
export type StoreKey = string | Iterable<number> | ArrayLike<number> | ArrayBuffer;
export interface ConnectionLimits {
    maxPendingIncoming?: number;
    maxPendingOutgoing?: number;
    maxEstablishedIncoming?: number;
    maxEstablishedOutgoing?: number;
    maxEstablishedPerPeer?: number;
    maxEstablishedTotal?: number;
}
export interface PeerAddress {
    known: string[];
    use_relay_fallback: boolean;
}
export interface AddressInfo {
    peers: Map<string, PeerAddress>;
    relays: string[];
}
export interface ClientAccess {
    useVaultDefault?: boolean;
    useVaultExceptions?: Map<VaultPath, boolean>;
    writeVaultDefault?: boolean;
    writeVaultExceptions?: Map<VaultPath, boolean>;
    cloneVaultDefault?: boolean;
    cloneVaultExceptions?: Map<VaultPath, boolean>;
    readStore?: boolean;
    writeStore?: boolean;
}
export interface Permissions {
    default?: ClientAccess;
    exceptions?: Map<VaultPath, ClientAccess>;
}
export interface NetworkConfig {
    requestTimeout?: Duration;
    connectionTimeout?: Duration;
    connectionsLimit?: ConnectionLimits;
    enableMdns?: boolean;
    enableRelay?: boolean;
    addresses?: AddressInfo;
    peerPermissions?: Map<string, Permissions>;
    permissionsDefault?: Permissions;
}
export interface Duration {
    millis: number;
    nanos: number;
}
export declare class Location {
    type: string;
    payload: Record<string, unknown>;
    constructor(type: string, payload: Record<string, unknown>);
    static generic(vault: VaultPath, record: RecordPath): Location;
    static counter(vault: VaultPath, counter: number): Location;
}
declare class ProcedureExecutor {
    procedureArgs: Record<string, unknown>;
    constructor(procedureArgs: Record<string, unknown>);
    generateSLIP10Seed(outputLocation: Location, sizeBytes?: number): Promise<Uint8Array>;
    deriveSLIP10(chain: number[], source: "Seed" | "Key", sourceLocation: Location, outputLocation: Location): Promise<Uint8Array>;
    recoverBIP39(mnemonic: string, outputLocation: Location, passphrase?: string): Promise<Uint8Array>;
    generateBIP39(outputLocation: Location, passphrase?: string): Promise<Uint8Array>;
    getEd25519PublicKey(privateKeyLocation: Location): Promise<Uint8Array>;
    signEd25519(privateKeyLocation: Location, msg: string): Promise<Uint8Array>;
}
export declare class Client {
    path: string;
    name: BytesDto;
    constructor(path: string, name: ClientPath);
    getVault(name: VaultPath): Vault;
    getStore(): Store;
}
export declare class Store {
    path: string;
    client: BytesDto;
    constructor(path: string, client: BytesDto);
    get(key: StoreKey): Promise<Uint8Array>;
    insert(key: StoreKey, value: number[], lifetime?: Duration): Promise<void>;
    remove(key: StoreKey): Promise<Uint8Array | null>;
}
export declare class Vault extends ProcedureExecutor {
    path: string;
    client: BytesDto;
    name: BytesDto;
    constructor(path: string, client: ClientPath, name: VaultPath);
    insert(recordPath: RecordPath, secret: number[]): Promise<void>;
    remove(location: Location): Promise<void>;
}
export declare class Stronghold {
    path: string;
    constructor(path: string, password: string);
    private reload;
    unload(): Promise<void>;
    loadClient(client: ClientPath): Promise<Client>;
    createClient(client: ClientPath): Promise<Client>;
    save(): Promise<void>;
}
export {};
