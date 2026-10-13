// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Store secrets and keys using the [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs)
 * encrypted database and secure runtime.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * The name of a Stronghold client, either as a UTF-8 string or as its raw byte representation.
 */
export type ClientPath =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer
/**
 * The path of a vault inside a client, either as a UTF-8 string or as its raw byte representation.
 */
export type VaultPath =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer
/**
 * The path of a record inside a vault, either as a UTF-8 string or as its raw byte representation.
 */
export type RecordPath =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer
/**
 * The key of a record in a client store. Note that the store commands read the
 * key as a string on the Rust side, so string keys are the safe choice.
 */
export type StoreKey =
  | string
  | Iterable<number>
  | ArrayLike<number>
  | ArrayBuffer

/**
 * The limits applied to the connections of a Stronghold peer-to-peer network.
 * See {@link NetworkConfig} for a note on how these definitions are used.
 */
export interface ConnectionLimits {
  /** The maximum number of incoming connections that can be pending at the same time. */
  maxPendingIncoming?: number
  /** The maximum number of outgoing connections that can be pending at the same time. */
  maxPendingOutgoing?: number
  /** The maximum number of established incoming connections. */
  maxEstablishedIncoming?: number
  /** The maximum number of established outgoing connections. */
  maxEstablishedOutgoing?: number
  /** The maximum number of established connections per peer. */
  maxEstablishedPerPeer?: number
  /** The maximum number of established connections, incoming and outgoing combined. */
  maxEstablishedTotal?: number
}

/**
 * The addresses on which a remote peer can be reached.
 * See {@link NetworkConfig} for a note on how these definitions are used.
 */
export interface PeerAddress {
  /** The addresses that are known for the peer, in the multiaddr format. */
  known: string[] // multiaddr
  /** Whether the peer may be reached through a relay when its known addresses cannot be used. */
  use_relay_fallback: boolean
}

/**
 * The address book of the peer-to-peer network.
 * See {@link NetworkConfig} for a note on how these definitions are used.
 */
export interface AddressInfo {
  /** The known addresses of each peer, keyed by the peer identifier. */
  peers: Map<string, PeerAddress>
  /** The identifiers of the peers that can be used as relays. */
  relays: string[] // peers
}

/**
 * The operations a remote peer is allowed to perform on a client.
 * See {@link NetworkConfig} for a note on how these definitions are used.
 */
export interface ClientAccess {
  /** The default permission to use the secrets stored in a vault. */
  useVaultDefault?: boolean
  /** Per-vault overrides of {@link ClientAccess.useVaultDefault}. */
  useVaultExceptions?: Map<VaultPath, boolean>
  /** The default permission to write secrets to a vault. */
  writeVaultDefault?: boolean
  /** Per-vault overrides of {@link ClientAccess.writeVaultDefault}. */
  writeVaultExceptions?: Map<VaultPath, boolean>
  /** The default permission to clone the secrets of a vault. */
  cloneVaultDefault?: boolean
  /** Per-vault overrides of {@link ClientAccess.cloneVaultDefault}. */
  cloneVaultExceptions?: Map<VaultPath, boolean>
  /** Whether the store of the client can be read. */
  readStore?: boolean
  /** Whether the store of the client can be written to. */
  writeStore?: boolean
}

/**
 * The access a remote peer is granted on the clients of a snapshot.
 * See {@link NetworkConfig} for a note on how these definitions are used.
 */
export interface Permissions {
  /** The access granted when no exception matches. */
  default?: ClientAccess
  /** The access granted for specific paths, overriding {@link Permissions.default}. */
  exceptions?: Map<VaultPath, ClientAccess>
}

/**
 * The configuration of the Stronghold peer-to-peer network.
 *
 * These definitions mirror the networking options of IOTA Stronghold. The plugin
 * does not currently expose a command that consumes them, so they are only useful
 * as type definitions.
 */
export interface NetworkConfig {
  /** The maximum time to wait for the response to an outbound request. */
  requestTimeout?: Duration
  /** The maximum time to wait when establishing a connection to a peer. */
  connectionTimeout?: Duration
  /** The limits applied to pending and established connections. */
  connectionsLimit?: ConnectionLimits
  /** Whether peers on the local network are discovered through mDNS. */
  enableMdns?: boolean
  /** Whether connections relayed by another peer are enabled. */
  enableRelay?: boolean
  /** The addresses of the known peers and of the available relays. */
  addresses?: AddressInfo
  /** The permissions granted to specific peers, keyed by the peer identifier. */
  peerPermissions?: Map<string, Permissions>
  /** The permissions granted to the peers that have no entry in {@link NetworkConfig.peerPermissions}. */
  permissionsDefault?: Permissions
}

/** A duration definition. */
export interface Duration {
  /** The number of whole seconds contained by this Duration. */
  secs: number
  /** The fractional part of this Duration, in nanoseconds. Must be greater or equal to 0 and smaller than 1e+9 (the max number of nanoseoncds in a second) */
  nanos: number
}

/**
 * A pointer to a record inside a vault, either addressed by its record path
 * ({@link Location.generic}) or by a counter ({@link Location.counter}).
 *
 * @since 2.0.0
 */
export class Location {
  /** The location kind, either `Generic` or `Counter`. */
  type: string
  /** The location data, holding the vault path and the record path or counter. */
  payload: Record<string, unknown>

  /**
   * Creates a location of the given kind. Prefer the {@link Location.generic}
   * and {@link Location.counter} helpers, which fill the payload for you.
   *
   * @example
   * ```typescript
   * import { Location } from '@tauri-apps/plugin-stronghold';
   * const location = new Location('Generic', { vault: 'my-vault', record: 'my-record' });
   * ```
   *
   * @param type The location kind, either `Generic` or `Counter`.
   * @param payload The data identifying the record inside the vault.
   */
  constructor(type: string, payload: Record<string, unknown>) {
    this.type = type
    this.payload = payload
  }

  /**
   * Creates a location addressing a record of a vault by its record path.
   *
   * @example
   * ```typescript
   * import { Location } from '@tauri-apps/plugin-stronghold';
   * const location = Location.generic('my-vault', 'my-record');
   * ```
   *
   * @param vault The path of the vault holding the record.
   * @param record The path of the record inside the vault.
   * @returns The location of the record.
   */
  static generic(vault: VaultPath, record: RecordPath): Location {
    return new Location('Generic', {
      vault,
      record
    })
  }

  /**
   * Creates a location addressing a record of a vault by a counter.
   *
   * @example
   * ```typescript
   * import { Location } from '@tauri-apps/plugin-stronghold';
   * const location = Location.counter('my-vault', 0);
   * ```
   *
   * @param vault The path of the vault holding the record.
   * @param counter The counter identifying the record inside the vault.
   * @returns The location of the record.
   */
  static counter(vault: VaultPath, counter: number): Location {
    return new Location('Counter', {
      vault,
      counter
    })
  }
}

class ProcedureExecutor {
  procedureArgs: Record<string, unknown>

  constructor(procedureArgs: Record<string, unknown>) {
    this.procedureArgs = procedureArgs
  }

  /**
   * Generate a SLIP10 seed for the given location.
   * @param outputLocation Location of the record where the seed will be stored.
   * @param sizeBytes The size in bytes of the SLIP10 seed.
   * @returns A promise resolving to the bytes returned by the procedure.
   */
  async generateSLIP10Seed(
    outputLocation: Location,
    sizeBytes?: number
  ): Promise<Uint8Array> {
    return await invoke<number[]>('plugin:stronghold|execute_procedure', {
      ...this.procedureArgs,
      procedure: {
        type: 'SLIP10Generate',
        payload: {
          output: outputLocation,
          sizeBytes
        }
      }
    }).then((n) => Uint8Array.from(n))
  }

  /**
   * Derive a SLIP10 private key using a seed or key. The derivation is always
   * performed on the Ed25519 curve.
   * @param chain The chain path.
   * @param source The source type, either 'Seed' or 'Key'.
   * @param sourceLocation The source location, must be the `outputLocation` of a previous call to `generateSLIP10Seed` or `deriveSLIP10`.
   * @param outputLocation Location of the record where the private key will be stored.
   * @returns A promise resolving to the bytes returned by the procedure.
   */
  async deriveSLIP10(
    chain: number[],
    source: 'Seed' | 'Key',
    sourceLocation: Location,
    outputLocation: Location
  ): Promise<Uint8Array> {
    return await invoke<number[]>('plugin:stronghold|execute_procedure', {
      ...this.procedureArgs,
      procedure: {
        type: 'SLIP10Derive',
        payload: {
          chain,
          input: {
            type: source,
            payload: sourceLocation
          },
          output: outputLocation
        }
      }
    }).then((n) => Uint8Array.from(n))
  }

  /**
   * Store a BIP39 mnemonic.
   * @param mnemonic The mnemonic string.
   * @param outputLocation The location of the record where the BIP39 mnemonic will be stored.
   * @param passphrase The optional mnemonic passphrase.
   * @returns A promise resolving to the bytes returned by the procedure.
   */
  async recoverBIP39(
    mnemonic: string,
    outputLocation: Location,
    passphrase?: string
  ): Promise<Uint8Array> {
    return await invoke<number[]>('plugin:stronghold|execute_procedure', {
      ...this.procedureArgs,
      procedure: {
        type: 'BIP39Recover',
        payload: {
          mnemonic,
          passphrase,
          output: outputLocation
        }
      }
    }).then((n) => Uint8Array.from(n))
  }

  /**
   * Generate a BIP39 seed. The mnemonic is generated in English.
   * @param outputLocation The location of the record where the BIP39 seed will be stored.
   * @param passphrase The optional mnemonic passphrase.
   * @returns A promise resolving to the bytes returned by the procedure.
   */
  async generateBIP39(
    outputLocation: Location,
    passphrase?: string
  ): Promise<Uint8Array> {
    return await invoke<number[]>('plugin:stronghold|execute_procedure', {
      ...this.procedureArgs,
      procedure: {
        type: 'BIP39Generate',
        payload: {
          output: outputLocation,
          passphrase
        }
      }
    }).then((n) => Uint8Array.from(n))
  }

  /**
   * Gets the Ed25519 public key of a SLIP10 private key.
   * @param privateKeyLocation The location of the private key. Must be the `outputLocation` of a previous call to `deriveSLIP10`.
   * @returns A promise resolving to the public key hex string.
   *
   * @since 2.0.0
   */
  async getEd25519PublicKey(privateKeyLocation: Location): Promise<Uint8Array> {
    return await invoke<number[]>('plugin:stronghold|execute_procedure', {
      ...this.procedureArgs,
      procedure: {
        type: 'PublicKey',
        payload: {
          type: 'Ed25519',
          privateKey: privateKeyLocation
        }
      }
    }).then((n) => Uint8Array.from(n))
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
    msg: string
  ): Promise<Uint8Array> {
    return await invoke<number[]>('plugin:stronghold|execute_procedure', {
      ...this.procedureArgs,
      procedure: {
        type: 'Ed25519Sign',
        payload: {
          privateKey: privateKeyLocation,
          msg
        }
      }
    }).then((n) => Uint8Array.from(n))
  }
}

/**
 * A client of a stronghold snapshot, owning a set of vaults and a key-value store.
 * Clients are obtained with {@link Stronghold.loadClient} and {@link Stronghold.createClient}.
 *
 * @since 2.0.0
 */
export class Client {
  /** The path of the snapshot file this client belongs to. */
  path: string
  /** The name identifying this client inside the snapshot. */
  name: ClientPath

  /**
   * Creates a client handle for a client that was already loaded or created.
   * Prefer {@link Stronghold.loadClient} and {@link Stronghold.createClient},
   * which also register the client on the Rust side.
   *
   * @example
   * ```typescript
   * import { Client } from '@tauri-apps/plugin-stronghold';
   * const client = new Client('/path/to/snapshot.hold', 'my-client');
   * ```
   *
   * @param path The path of the snapshot file the client belongs to.
   * @param name The name identifying the client inside the snapshot.
   */
  constructor(path: string, name: ClientPath) {
    this.path = path
    this.name = name
  }

  /**
   * Gets a handle to the vault with the given name. The vault is created on the
   * Rust side when the first secret is written to it.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * const vault = client.getVault('my-vault');
   * ```
   *
   * @param name The path of the vault.
   * @returns The vault handle.
   */
  getVault(name: VaultPath): Vault {
    return new Vault(this.path, this.name, name)
  }

  /**
   * Gets a handle to the key-value store of this client.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * const store = client.getStore();
   * ```
   *
   * @returns The store handle.
   */
  getStore(): Store {
    return new Store(this.path, this.name)
  }
}

/**
 * The key-value store of a {@link Client}. Unlike a {@link Vault}, the values
 * stored here can be read back directly.
 *
 * @since 2.0.0
 */
export class Store {
  /** The path of the snapshot file this store belongs to. */
  path: string
  /** The name of the client owning this store. */
  client: ClientPath

  /**
   * Creates a store handle for a client that was already loaded or created.
   * Prefer {@link Client.getStore}.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-stronghold';
   * const store = new Store('/path/to/snapshot.hold', 'my-client');
   * ```
   *
   * @param path The path of the snapshot file the store belongs to.
   * @param client The name of the client owning the store.
   */
  constructor(path: string, client: ClientPath) {
    this.path = path
    this.client = client
  }

  /**
   * Reads the value of a record of this store.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * const value = await client.getStore().get('my-key');
   * ```
   *
   * @param key The key of the record.
   * @returns A promise resolving to the stored value, or `null` if the key does not exist.
   */
  async get(key: StoreKey): Promise<Uint8Array | null> {
    return await invoke<number[] | null>('plugin:stronghold|get_store_record', {
      snapshotPath: this.path,
      client: this.client,
      key
    }).then((v) => v && Uint8Array.from(v))
  }

  /**
   * Inserts a record in this store, replacing the previous value of the key.
   * Note that the snapshot is only persisted when {@link Stronghold.save} is called.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * const data = Array.from(new TextEncoder().encode('Hello, World!'));
   * await client.getStore().insert('my-key', data);
   * await stronghold.save();
   * ```
   *
   * @param key The key of the record.
   * @param value The value of the record, as an array of bytes.
   * @param lifetime The optional duration after which the record expires.
   */
  async insert(
    key: StoreKey,
    value: number[],
    lifetime?: Duration
  ): Promise<void> {
    await invoke('plugin:stronghold|save_store_record', {
      snapshotPath: this.path,
      client: this.client,
      key,
      value,
      lifetime
    })
  }

  /**
   * Deletes a record from this store.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * await client.getStore().remove('my-key');
   * ```
   *
   * @param key The key of the record.
   * @returns A promise resolving to the deleted value, or `null` if the key did not exist.
   */
  async remove(key: StoreKey): Promise<Uint8Array | null> {
    return await invoke<number[] | null>(
      'plugin:stronghold|remove_store_record',
      {
        snapshotPath: this.path,
        client: this.client,
        key
      }
    ).then((v) => v && Uint8Array.from(v))
  }
}

/**
 * A key-value storage that allows create, update and delete operations.
 * It does not allow reading the data, so one of the procedures must be used to manipulate
 * the stored data, allowing secure storage of secrets.
 *
 * @since 2.0.0
 */
export class Vault extends ProcedureExecutor {
  /** The path of the snapshot file this vault belongs to. */
  path: string
  /** The name of the client owning this vault. */
  client: ClientPath
  /** The path identifying this vault inside the client. */
  name: VaultPath

  /**
   * Creates a vault handle for a client that was already loaded or created.
   * Prefer {@link Client.getVault}.
   *
   * @example
   * ```typescript
   * import { Vault } from '@tauri-apps/plugin-stronghold';
   * const vault = new Vault('/path/to/snapshot.hold', 'my-client', 'my-vault');
   * ```
   *
   * @param path The path of the snapshot file the vault belongs to.
   * @param client The name of the client owning the vault.
   * @param name The path identifying the vault inside the client.
   */
  constructor(path: string, client: ClientPath, name: VaultPath) {
    super({
      snapshotPath: path,
      client,
      vault: name
    })
    this.path = path
    this.client = client
    this.name = name
  }

  /**
   * Writes a secret to this vault. Note that the snapshot is only persisted
   * when {@link Stronghold.save} is called.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * const secret = Array.from(new TextEncoder().encode('secret value'));
   * await client.getVault('my-vault').insert('my-record', secret);
   * await stronghold.save();
   * ```
   *
   * @param recordPath The path of the record inside this vault.
   * @param secret The secret to store, as an array of bytes.
   */
  async insert(recordPath: RecordPath, secret: number[]): Promise<void> {
    await invoke('plugin:stronghold|save_secret', {
      snapshotPath: this.path,
      client: this.client,
      vault: this.name,
      recordPath,
      secret
    })
  }

  /**
   * Deletes a secret from this vault. Only the record path of the given
   * location is used, the vault is always this one.
   *
   * @example
   * ```typescript
   * import { Stronghold, Location } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * await client
   *   .getVault('my-vault')
   *   .remove(Location.generic('my-vault', 'my-record'));
   * ```
   *
   * @param location The location of the record to delete.
   */
  async remove(location: Location): Promise<void> {
    await invoke('plugin:stronghold|remove_secret', {
      snapshotPath: this.path,
      client: this.client,
      vault: this.name,
      recordPath: location.payload.record
    })
  }
}

/**
 * A representation of an access to a stronghold.
 *
 * @since 2.0.0
 */
export class Stronghold {
  /** The path of the snapshot file backing this stronghold. */
  path: string

  /**
   * Creates a handle to the stronghold initialized for the given snapshot path.
   * @param path The path of the snapshot file.
   */
  private constructor(path: string) {
    this.path = path
  }

  /**
   * Load the snapshot if it exists (password must match), or start a fresh stronghold instance otherwise.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * import { appDataDir } from '@tauri-apps/api/path';
   * const stronghold = await Stronghold.load(`${await appDataDir()}/vault.hold`, 'password');
   * ```
   *
   * @param path The path of the snapshot file.
   * @param password The password used to encrypt and decrypt the snapshot.
   * @returns A promise resolving to the stronghold instance.
   */
  static async load(path: string, password: string): Promise<Stronghold> {
    return await invoke('plugin:stronghold|initialize', {
      snapshotPath: path,
      password
    }).then(() => new Stronghold(path))
  }

  /**
   * Saves the snapshot and removes this instance from the cache.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * await stronghold.unload();
   * ```
   */
  async unload(): Promise<void> {
    await invoke('plugin:stronghold|destroy', {
      snapshotPath: this.path
    })
  }

  /**
   * Loads an existing client from the snapshot. The promise rejects if the
   * client does not exist in the snapshot or was already loaded.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.loadClient('my-client');
   * ```
   *
   * @param client The name of the client.
   * @returns A promise resolving to the loaded client.
   */
  async loadClient(client: ClientPath): Promise<Client> {
    return await invoke('plugin:stronghold|load_client', {
      snapshotPath: this.path,
      client
    }).then(() => new Client(this.path, client))
  }

  /**
   * Creates a new empty client on this stronghold.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * let client;
   * try {
   *   client = await stronghold.loadClient('my-client');
   * } catch {
   *   client = await stronghold.createClient('my-client');
   * }
   * ```
   *
   * @param client The name of the client.
   * @returns A promise resolving to the created client.
   */
  async createClient(client: ClientPath): Promise<Client> {
    return await invoke('plugin:stronghold|create_client', {
      snapshotPath: this.path,
      client
    }).then(() => new Client(this.path, client))
  }

  /**
   * Persists the stronghold state to the snapshot.
   *
   * @example
   * ```typescript
   * import { Stronghold } from '@tauri-apps/plugin-stronghold';
   * const stronghold = await Stronghold.load('/path/to/snapshot.hold', 'password');
   * const client = await stronghold.createClient('my-client');
   * await client.getStore().insert('my-key', [1, 2, 3]);
   * await stronghold.save();
   * ```
   */
  async save(): Promise<void> {
    await invoke('plugin:stronghold|save', {
      snapshotPath: this.path
    })
  }
}
