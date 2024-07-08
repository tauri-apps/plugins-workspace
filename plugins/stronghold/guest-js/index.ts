// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";

type BytesDto = string | number[];
export type ClientPath =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer;
export type VaultPath =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer;
export type RecordPath =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer;
export type StoreKey =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer;

function toBytesDto(
  v: ClientPath | VaultPath | RecordPath | StoreKey,
): string | number[] {
  if (typeof v === "string") {
    return v;
  }
  return Array.from(v instanceof ArrayBuffer ? new Uint8Array(v) : v);
}

export interface ConnectionLimits {
  maxPendingIncoming?: number;
  maxPendingOutgoing?: number;
  maxEstablishedIncoming?: number;
  maxEstablishedOutgoing?: number;
  maxEstablishedPerPeer?: number;
  maxEstablishedTotal?: number;
}

export interface PeerAddress {
  known: string[]; // multiaddr
  use_relay_fallback: boolean;
}

export interface AddressInfo {
  peers: Map<string, PeerAddress>;
  relays: string[]; // peers
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

/** A duration definition. */
export interface Duration {
  /** The number of whole seconds contained by this Duration. */
  secs: number;
  /** The fractional part of this Duration, in nanoseconds. Must be greater or equal to 0 and smaller than 1e+9 (the max number of nanoseoncds in a second) */
  nanos: number;
}

export class Location {
  type: string;
  payload: Record<string, unknown>;

  constructor(type: string, payload: Record<string, unknown>) {
    this.type = type;
    this.payload = payload;
  }

  static generic(vault: VaultPath, record: RecordPath): Location {
    return new Location("Generic", {
      vault: toBytesDto(vault),
      record: toBytesDto(record),
    });
  }

  static counter(vault: VaultPath, counter: number): Location {
    return new Location("Counter", {
      vault: toBytesDto(vault),
      counter,
    });
  }
}

class ProcedureExecutor {
  procedureArgs: Record<string, unknown>;

  constructor(procedureArgs: Record<string, unknown>) {
    this.procedureArgs = procedureArgs;
  }

  /**
   * Generate a SLIP10 seed for the given location.
   * @param outputLocation Location of the record where the seed will be stored.
   * @param sizeBytes The size in bytes of the SLIP10 seed.
   * @param hint The record hint.
   * @returns
   */
  async generateSLIP10Seed(
    outputLocation: Location,
    sizeBytes?: number,
  ): Promise<Uint8Array> {
    return await invoke<number[]>("plugin:stronghold|execute_procedure", {
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

  /**
   * Derive a SLIP10 private key using a seed or key.
   * @param chain The chain path.
   * @param source The source type, either 'Seed' or 'Key'.
   * @param sourceLocation The source location, must be the `outputLocation` of a previous call to `generateSLIP10Seed` or `deriveSLIP10`.
   * @param outputLocation Location of the record where the private key will be stored.
   * @param hint The record hint.
   * @returns
   */
  async deriveSLIP10(
    chain: number[],
    source: "Seed" | "Key",
    sourceLocation: Location,
    outputLocation: Location,
  ): Promise<Uint8Array> {
    return await invoke<number[]>("plugin:stronghold|execute_procedure", {
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

  /**
   * Store a BIP39 mnemonic.
   * @param mnemonic The mnemonic string.
   * @param outputLocation The location of the record where the BIP39 mnemonic will be stored.
   * @param passphrase The optional mnemonic passphrase.
   * @param hint The record hint.
   * @returns
   */
  async recoverBIP39(
    mnemonic: string,
    outputLocation: Location,
    passphrase?: string,
  ): Promise<Uint8Array> {
    return await invoke<number[]>("plugin:stronghold|execute_procedure", {
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

  /**
   * Generate a BIP39 seed.
   * @param outputLocation The location of the record where the BIP39 seed will be stored.
   * @param passphrase The optional mnemonic passphrase.
   * @param hint The record hint.
   * @returns
   */
  async generateBIP39(
    outputLocation: Location,
    passphrase?: string,
  ): Promise<Uint8Array> {
    return await invoke<number[]>("plugin:stronghold|execute_procedure", {
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

  /**
   * Gets the Ed25519 public key of a SLIP10 private key.
   * @param privateKeyLocation The location of the private key. Must be the `outputLocation` of a previous call to `deriveSLIP10`.
   * @returns A promise resolving to the public key hex string.
   *
   * @since 2.0.0
   */
  async getEd25519PublicKey(privateKeyLocation: Location): Promise<Uint8Array> {
    return await invoke<number[]>("plugin:stronghold|execute_procedure", {
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

  /**
   * Creates a Ed25519 signature from a private key.
   * @param privateKeyLocation The location of the record where the private key is stored. Must be the `outputLocation` of a previous call to `deriveSLIP10`.
   * @param msg The message to sign.
   * @returns A promise resolving to the signature hex string.
   *
   * @since 2.0.0
   */
  async signEd25519(
    privateKeyLocation: Location,
    msg: string,
  ): Promise<Uint8Array> {
    return await invoke<number[]>("plugin:stronghold|execute_procedure", {
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

export class Client {
  path: string;
  name: BytesDto;

  constructor(path: string, name: ClientPath) {
    this.path = path;
    this.name = toBytesDto(name);
  }

  /**
   * Get a vault by name.
   * @param name
   * @param flags
   * @returns
   */
  getVault(name: VaultPath): Vault {
    return new Vault(this.path, this.name, toBytesDto(name));
  }

  getStore(): Store {
    return new Store(this.path, this.name);
  }
}

export class Store {
  path: string;
  client: BytesDto;

  constructor(path: string, client: BytesDto) {
    this.path = path;
    this.client = client;
  }

  async get(key: StoreKey): Promise<Uint8Array | null> {
    return await invoke<number[] | null>("plugin:stronghold|get_store_record", {
      snapshotPath: this.path,
      client: this.client,
      key: toBytesDto(key),
    }).then((v) => v && Uint8Array.from(v));
  }

  async insert(
    key: StoreKey,
    value: number[],
    lifetime?: Duration,
  ): Promise<void> {
    await invoke("plugin:stronghold|save_store_record", {
      snapshotPath: this.path,
      client: this.client,
      key: toBytesDto(key),
      value,
      lifetime,
    });
  }

  async remove(key: StoreKey): Promise<Uint8Array | null> {
    return await invoke<number[] | null>(
      "plugin:stronghold|remove_store_record",
      {
        snapshotPath: this.path,
        client: this.client,
        key: toBytesDto(key),
      },
    ).then((v) => v && Uint8Array.from(v));
  }
}

/**
 * A key-value storage that allows create, update and delete operations.
 * It does not allow reading the data, so one of the procedures must be used to manipulate
 * the stored data, allowing secure storage of secrets.
 */
export class Vault extends ProcedureExecutor {
  /** The vault path. */
  path: string;
  client: BytesDto;
  /** The vault name. */
  name: BytesDto;

  constructor(path: string, client: ClientPath, name: VaultPath) {
    super({
      snapshotPath: path,
      client,
      vault: name,
    });
    this.path = path;
    this.client = toBytesDto(client);
    this.name = toBytesDto(name);
  }

  /**
   * Insert a record to this vault.
   * @param location The record location.
   * @param record  The record data.
   * @param recordHint The record hint.
   * @returns
   */
  async insert(recordPath: RecordPath, secret: number[]): Promise<void> {
    await invoke("plugin:stronghold|save_secret", {
      snapshotPath: this.path,
      client: this.client,
      vault: this.name,
      recordPath: toBytesDto(recordPath),
      secret,
    });
  }

  /**
   * Remove a record from the vault.
   * @param location The record location.
   * @param gc Whether to additionally perform the gargage collection or not.
   * @returns
   */
  async remove(location: Location): Promise<void> {
    await invoke("plugin:stronghold|remove_secret", {
      snapshotPath: this.path,
      client: this.client,
      vault: this.name,
      recordPath: location.payload.record,
    });
  }
}

/**
 * A representation of an access to a stronghold.
 */
export class Stronghold {
  path: string;

  /**
   * Initializes a stronghold.
   * If the snapshot path located at `path` exists, the password must match.
   * @param path
   * @param password
   */
  private constructor(path: string) {
    this.path = path;
  }

  /**
   * Load the snapshot if it exists (password must match), or start a fresh stronghold instance otherwise.
   * @param password
   * @returns
   */
  static async load(path: string, password: string): Promise<Stronghold> {
    return await invoke("plugin:stronghold|initialize", {
      snapshotPath: path,
      password,
    }).then(() => new Stronghold(path));
  }

  /**
   * Remove this instance from the cache.
   */
  async unload(): Promise<void> {
    await invoke("plugin:stronghold|destroy", {
      snapshotPath: this.path,
    });
  }

  async loadClient(client: ClientPath): Promise<Client> {
    return await invoke("plugin:stronghold|load_client", {
      snapshotPath: this.path,
      client: toBytesDto(client),
    }).then(() => new Client(this.path, client));
  }

  async createClient(client: ClientPath): Promise<Client> {
    return await invoke("plugin:stronghold|create_client", {
      snapshotPath: this.path,
      client: toBytesDto(client),
    }).then(() => new Client(this.path, client));
  }

  /**
   * Persists the stronghold state to the snapshot.
   * @returns
   */
  async save(): Promise<void> {
    await invoke("plugin:stronghold|save", {
      snapshotPath: this.path,
    });
  }
}
