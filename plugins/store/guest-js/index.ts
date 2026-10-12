// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Simple, persistent key-value store.
 *
 * A store is persisted to a file inside the application data directory and is shared with the
 * Rust side of the application, which can read and write the same store through its own API.
 *
 * @module
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import { invoke, Resource } from '@tauri-apps/api/core'

interface ChangePayload<T> {
  path: string
  resourceId?: number
  key: string
  value: T
  exists: boolean
}

/**
 * Options to create a store
 */
export type StoreOptions = {
  /**
   * Default value of the store
   */
  defaults?: { [key: string]: unknown }
  /**
   * Auto save on modification with debounce duration in milliseconds, it's 100ms by default, pass in `false` to disable it
   */
  autoSave?: boolean | number
  /**
   * Name of a serialize function registered in the rust side plugin builder
   */
  serializeFnName?: string
  /**
   * Name of a deserialize function registered in the rust side plugin builder
   */
  deserializeFnName?: string
  /**
   * Force create a new store with default values even if it already exists.
   */
  createNew?: boolean
  /**
   * When creating the store, override the store with the on-disk state if it exists, ignoring defaults
   */
  overrideDefaults?: boolean
}

/**
 * Create a new Store or load the existing store with the path.
 *
 * If the file at the given path does not exist yet, the store is created in memory with the
 * configured defaults and the file is only written on the first save.
 *
 * @example
 * ```typescript
 * import { load } from '@tauri-apps/plugin-store';
 * const store = await load('store.json');
 * ```
 *
 * @param path Path to save the store in `app_data_dir`
 * @param options Store configuration options
 * @returns A promise resolving to the loaded store.
 *
 * @since 2.1.0
 */
export async function load(
  path: string,
  options?: StoreOptions
): Promise<Store> {
  return await Store.load(path, options)
}

/**
 * Gets an already loaded store.
 *
 * If the store is not loaded, returns `null`. In this case you must {@link Store.load load} it.
 *
 * This function is more useful when you already know the store is loaded
 * and just need to access its instance. Prefer {@link Store.load} otherwise.
 *
 * @example
 * ```typescript
 * import { getStore } from '@tauri-apps/plugin-store';
 * const store = await getStore('store.json');
 * ```
 *
 * @param path Path of the store.
 * @returns A promise resolving to the store instance, or `null` if it is not loaded.
 *
 * @since 2.1.0
 */
export async function getStore(path: string): Promise<Store | null> {
  return await Store.get(path)
}

/**
 * A lazy loaded key-value store persisted by the backend layer.
 *
 * The underlying {@linkcode Store} is only created or loaded when one of the methods of this
 * class is called for the first time, and every call afterwards reuses that same instance.
 *
 * @since 2.1.0
 */
export class LazyStore implements IStore {
  private _store?: Promise<Store>

  private get store(): Promise<Store> {
    if (!this._store) {
      this._store = load(this.path, this.options)
    }
    return this._store
  }

  /**
   * Creates a handle to the store at the given path without loading it yet.
   *
   * Note that the options are not applied if someone else already created the store
   *
   * @param path Path to save the store in `app_data_dir`
   * @param options Store configuration options
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * ```
   */
  constructor(
    private readonly path: string,
    private readonly options?: StoreOptions
  ) {}

  /**
   * Init/load the store if it's not loaded already
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * await store.init();
   * ```
   */
  async init(): Promise<void> {
    await this.store
  }

  /**
   * Inserts a key-value pair into the store, loading it first if needed.
   *
   * Delegates to {@linkcode Store.set} on the underlying store.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * await store.set('some-key', { value: 5 });
   * ```
   *
   * @param key The key to insert the value at.
   * @param value The value to store, which must be serializable to JSON.
   */
  async set(key: string, value: unknown): Promise<void> {
    return (await this.store).set(key, value)
  }

  /**
   * Returns the value for the given `key` or `undefined` if the key does not exist.
   *
   * Delegates to {@linkcode Store.get} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const value = await store.get<{ value: number }>('some-key');
   * ```
   *
   * @param key The key to read the value of.
   * @returns A promise resolving to the stored value, or `undefined` if the key does not exist.
   */
  async get<T>(key: string): Promise<T | undefined> {
    return (await this.store).get<T>(key)
  }

  /**
   * Returns `true` if the given `key` exists in the store.
   *
   * Delegates to {@linkcode Store.has} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const exists = await store.has('some-key');
   * ```
   *
   * @param key The key to check.
   * @returns A promise resolving to `true` if the key exists in the store.
   */
  async has(key: string): Promise<boolean> {
    return (await this.store).has(key)
  }

  /**
   * Removes a key-value pair from the store.
   *
   * Delegates to {@linkcode Store.delete} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const removed = await store.delete('some-key');
   * ```
   *
   * @param key The key to remove.
   * @returns A promise resolving to `true` if the key existed and was removed.
   */
  async delete(key: string): Promise<boolean> {
    return (await this.store).delete(key)
  }

  /**
   * Clears the store, removing all key-value pairs.
   *
   * Note: To clear the storage and reset it to its `default` value, use {@linkcode reset} instead.
   * Delegates to {@linkcode Store.clear} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * await store.clear();
   * ```
   */
  async clear(): Promise<void> {
    await (await this.store).clear()
  }

  /**
   * Resets the store to its `default` value.
   *
   * If no default value has been set, this method behaves identical to {@linkcode clear}.
   * Delegates to {@linkcode Store.reset} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json', { defaults: { 'some-key': 0 } });
   * await store.reset();
   * ```
   */
  async reset(): Promise<void> {
    await (await this.store).reset()
  }

  /**
   * Returns a list of all keys in the store.
   *
   * Delegates to {@linkcode Store.keys} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const keys = await store.keys();
   * ```
   *
   * @returns A promise resolving to the list of keys, in arbitrary order.
   */
  async keys(): Promise<string[]> {
    return (await this.store).keys()
  }

  /**
   * Returns a list of all values in the store.
   *
   * Delegates to {@linkcode Store.values} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const values = await store.values();
   * ```
   *
   * @returns A promise resolving to the list of values, in arbitrary order.
   */
  async values<T>(): Promise<T[]> {
    return (await this.store).values<T>()
  }

  /**
   * Returns a list of all entries in the store.
   *
   * Delegates to {@linkcode Store.entries} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const entries = await store.entries();
   * ```
   *
   * @returns A promise resolving to the list of key-value pairs, in arbitrary order.
   */
  async entries<T>(): Promise<Array<[key: string, value: T]>> {
    return (await this.store).entries<T>()
  }

  /**
   * Returns the number of key-value pairs in the store.
   *
   * Delegates to {@linkcode Store.length} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const length = await store.length();
   * ```
   *
   * @returns A promise resolving to the number of key-value pairs in the store.
   */
  async length(): Promise<number> {
    return (await this.store).length()
  }

  /**
   * Attempts to load the on-disk state at the store's `path` into memory.
   *
   * Delegates to {@linkcode Store.reload} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * await store.reload({ ignoreDefaults: true });
   * ```
   *
   * @param options Options to change how the on-disk state is merged into the store.
   */
  async reload(options?: ReloadOptions): Promise<void> {
    await (await this.store).reload(options)
  }

  /**
   * Saves the store to disk at the store's `path`.
   *
   * Delegates to {@linkcode Store.save} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * await store.save();
   * ```
   */
  async save(): Promise<void> {
    await (await this.store).save()
  }

  /**
   * Listen to changes on a store key.
   *
   * Delegates to {@linkcode Store.onKeyChange} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const unlisten = await store.onKeyChange<{ value: number }>('some-key', (value) => {
   *   console.log(value);
   * });
   * ```
   *
   * @param key The key to watch for changes.
   * @param cb Callback invoked with the new value, or `undefined` when the key was removed.
   * @returns A promise resolving to a function to unlisten to the event.
   */
  async onKeyChange<T>(
    key: string,
    cb: (value: T | undefined) => void
  ): Promise<UnlistenFn> {
    return (await this.store).onKeyChange<T>(key, cb)
  }

  /**
   * Listen to changes on the store.
   *
   * Delegates to {@linkcode Store.onChange} on the underlying store, loading it first if needed.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * const unlisten = await store.onChange<{ value: number }>((key, value) => {
   *   console.log(key, value);
   * });
   * ```
   *
   * @param cb Callback invoked with the changed key and its new value, which is `undefined` when the key was removed.
   * @returns A promise resolving to a function to unlisten to the event.
   */
  async onChange<T>(
    cb: (key: string, value: T | undefined) => void
  ): Promise<UnlistenFn> {
    return (await this.store).onChange<T>(cb)
  }

  /**
   * Close the store and cleans up this resource from memory.
   * **You should not call any method on this object anymore and should drop any reference to it.**
   *
   * Delegates to {@linkcode Store.close} on the underlying store.
   * If the store was never loaded, this method does nothing.
   *
   * @example
   * ```typescript
   * import { LazyStore } from '@tauri-apps/plugin-store';
   * const store = new LazyStore('store.json');
   * await store.close();
   * ```
   */
  async close(): Promise<void> {
    if (this._store) {
      await (await this._store).close()
    }
  }
}

/**
 * A key-value store persisted by the backend layer.
 *
 * The values are kept in memory and written to the store's file on {@linkcode Store.save},
 * and automatically after every modification unless auto save is disabled with
 * {@linkcode StoreOptions.autoSave}.
 *
 * @since 2.0.0
 */
export class Store extends Resource implements IStore {
  private constructor(rid: number) {
    super(rid)
  }

  /**
   * Create a new Store or load the existing store with the path.
   *
   * If the file at the given path does not exist yet, the store is created in memory with the
   * configured defaults and the file is only written on the first save.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * ```
   *
   * @param path Path to save the store in `app_data_dir`
   * @param options Store configuration options
   * @returns A promise resolving to the loaded store.
   */
  static async load(path: string, options?: StoreOptions): Promise<Store> {
    const rid = await invoke<number>('plugin:store|load', {
      path,
      options
    })
    return new Store(rid)
  }

  /**
   * Gets an already loaded store.
   *
   * If the store is not loaded, returns `null`. In this case you must {@link Store.load load} it.
   *
   * This function is more useful when you already know the store is loaded
   * and just need to access its instance. Prefer {@link Store.load} otherwise.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * let store = await Store.get('store.json');
   * if (!store) {
   *   store = await Store.load('store.json');
   * }
   * ```
   *
   * @param path Path of the store.
   * @returns A promise resolving to the store instance, or `null` if it is not loaded.
   */
  static async get(path: string): Promise<Store | null> {
    return await invoke<number | null>('plugin:store|get_store', { path }).then(
      (rid) => (rid ? new Store(rid) : null)
    )
  }

  /**
   * Inserts a key-value pair into the store.
   *
   * A change event is emitted for the key and, unless auto save is disabled, the store is
   * scheduled to be written to disk.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * await store.set('some-key', { value: 5 });
   * ```
   *
   * @param key The key to insert the value at.
   * @param value The value to store, which must be serializable to JSON.
   */
  async set(key: string, value: unknown): Promise<void> {
    await invoke('plugin:store|set', {
      rid: this.rid,
      key,
      value
    })
  }

  /**
   * Returns the value for the given `key` or `undefined` if the key does not exist.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const value = await store.get<{ value: number }>('some-key');
   * ```
   *
   * @param key The key to read the value of.
   * @returns A promise resolving to the stored value, or `undefined` if the key does not exist.
   */
  async get<T>(key: string): Promise<T | undefined> {
    const [value, exists] = await invoke<[T, boolean]>('plugin:store|get', {
      rid: this.rid,
      key
    })
    return exists ? value : undefined
  }

  /**
   * Returns `true` if the given `key` exists in the store.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const exists = await store.has('some-key');
   * ```
   *
   * @param key The key to check.
   * @returns A promise resolving to `true` if the key exists in the store.
   */
  async has(key: string): Promise<boolean> {
    return await invoke('plugin:store|has', {
      rid: this.rid,
      key
    })
  }

  /**
   * Removes a key-value pair from the store.
   *
   * A change event is emitted when the key existed and, unless auto save is disabled, the store
   * is scheduled to be written to disk.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const removed = await store.delete('some-key');
   * ```
   *
   * @param key The key to remove.
   * @returns A promise resolving to `true` if the key existed and was removed.
   */
  async delete(key: string): Promise<boolean> {
    return await invoke('plugin:store|delete', {
      rid: this.rid,
      key
    })
  }

  /**
   * Clears the store, removing all key-value pairs.
   *
   * Note: To clear the storage and reset it to its `default` value, use {@linkcode reset} instead.
   * A change event is emitted for every removed key.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * await store.clear();
   * ```
   */
  async clear(): Promise<void> {
    await invoke('plugin:store|clear', { rid: this.rid })
  }

  /**
   * Resets the store to its `default` value.
   *
   * If no default value has been set, this method behaves identical to {@linkcode clear}.
   * A change event is emitted for every key whose value changed.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json', { defaults: { 'some-key': 0 } });
   * await store.reset();
   * ```
   */
  async reset(): Promise<void> {
    await invoke('plugin:store|reset', { rid: this.rid })
  }

  /**
   * Returns a list of all keys in the store.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const keys = await store.keys();
   * ```
   *
   * @returns A promise resolving to the list of keys, in arbitrary order.
   */
  async keys(): Promise<string[]> {
    return await invoke('plugin:store|keys', { rid: this.rid })
  }

  /**
   * Returns a list of all values in the store.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const values = await store.values();
   * ```
   *
   * @returns A promise resolving to the list of values, in arbitrary order.
   */
  async values<T>(): Promise<T[]> {
    return await invoke('plugin:store|values', { rid: this.rid })
  }

  /**
   * Returns a list of all entries in the store.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const entries = await store.entries();
   * ```
   *
   * @returns A promise resolving to the list of key-value pairs, in arbitrary order.
   */
  async entries<T>(): Promise<Array<[key: string, value: T]>> {
    return await invoke('plugin:store|entries', { rid: this.rid })
  }

  /**
   * Returns the number of key-value pairs in the store.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const length = await store.length();
   * ```
   *
   * @returns A promise resolving to the number of key-value pairs in the store.
   */
  async length(): Promise<number> {
    return await invoke('plugin:store|length', { rid: this.rid })
  }

  /**
   * Attempts to load the on-disk state at the store's `path` into memory.
   *
   * This method is useful if the on-disk state was edited by the user and you want to synchronize the changes.
   *
   * Note:
   *   - This method loads the data and merges it with the current store,
   *     this behavior will be changed to resetting to default first and then merging with the on-disk state in v3,
   *     to fully match the store with the on-disk state, set {@linkcode ReloadOptions | ignoreDefaults} to `true`
   *   - This method does not emit change events.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * await store.reload({ ignoreDefaults: true });
   * ```
   *
   * @param options Options to change how the on-disk state is merged into the store.
   */
  async reload(options?: ReloadOptions): Promise<void> {
    await invoke('plugin:store|reload', { rid: this.rid, ...options })
  }

  /**
   * Saves the store to disk at the store's `path`.
   *
   * Any pending auto save is cancelled, so the store is written exactly once by this call.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json', { autoSave: false });
   * await store.set('some-key', { value: 5 });
   * await store.save();
   * ```
   */
  async save(): Promise<void> {
    await invoke('plugin:store|save', { rid: this.rid })
  }

  /**
   * Listen to changes on a store key.
   *
   * The callback is only invoked for changes made to this store instance.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const unlisten = await store.onKeyChange<{ value: number }>('some-key', (value) => {
   *   console.log(value);
   * });
   * ```
   *
   * @param key The key to watch for changes.
   * @param cb Callback invoked with the new value, or `undefined` when the key was removed.
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @since 2.0.0
   */
  async onKeyChange<T>(
    key: string,
    cb: (value: T | undefined) => void
  ): Promise<UnlistenFn> {
    return await listen<ChangePayload<T>>('store://change', (event) => {
      if (event.payload.resourceId === this.rid && event.payload.key === key) {
        cb(event.payload.exists ? event.payload.value : undefined)
      }
    })
  }

  /**
   * Listen to changes on the store.
   *
   * The callback is only invoked for changes made to this store instance.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/plugin-store';
   * const store = await Store.load('store.json');
   * const unlisten = await store.onChange<{ value: number }>((key, value) => {
   *   console.log(key, value);
   * });
   * ```
   *
   * @param cb Callback invoked with the changed key and its new value, which is `undefined` when the key was removed.
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @since 2.0.0
   */
  async onChange<T>(
    cb: (key: string, value: T | undefined) => void
  ): Promise<UnlistenFn> {
    return await listen<ChangePayload<T>>('store://change', (event) => {
      if (event.payload.resourceId === this.rid) {
        cb(
          event.payload.key,
          event.payload.exists ? event.payload.value : undefined
        )
      }
    })
  }
}

interface IStore {
  /**
   * Inserts a key-value pair into the store.
   *
   * @param key
   * @param value
   * @returns
   */
  set(key: string, value: unknown): Promise<void>

  /**
   * Returns the value for the given `key` or `undefined` if the key does not exist.
   *
   * @param key
   * @returns
   */
  get<T>(key: string): Promise<T | undefined>

  /**
   * Returns `true` if the given `key` exists in the store.
   *
   * @param key
   * @returns
   */
  has(key: string): Promise<boolean>

  /**
   * Removes a key-value pair from the store.
   *
   * @param key
   * @returns
   */
  delete(key: string): Promise<boolean>

  /**
   * Clears the store, removing all key-value pairs.
   *
   * Note: To clear the storage and reset it to its `default` value, use {@linkcode reset} instead.
   * @returns
   */
  clear(): Promise<void>

  /**
   * Resets the store to its `default` value.
   *
   * If no default value has been set, this method behaves identical to {@linkcode clear}.
   * @returns
   */
  reset(): Promise<void>

  /**
   * Returns a list of all keys in the store.
   *
   * @returns
   */
  keys(): Promise<string[]>

  /**
   * Returns a list of all values in the store.
   *
   * @returns
   */
  values<T>(): Promise<T[]>

  /**
   * Returns a list of all entries in the store.
   *
   * @returns
   */
  entries<T>(): Promise<Array<[key: string, value: T]>>

  /**
   * Returns the number of key-value pairs in the store.
   *
   * @returns
   */
  length(): Promise<number>

  /**
   * Attempts to load the on-disk state at the store's `path` into memory.
   *
   * This method is useful if the on-disk state was edited by the user and you want to synchronize the changes.
   *
   * Note:
   *   - This method loads the data and merges it with the current store,
   *     this behavior will be changed to resetting to default first and then merging with the on-disk state in v3,
   *     to fully match the store with the on-disk state, set {@linkcode ReloadOptions | ignoreDefaults} to `true`
   *   - This method does not emit change events.
   *
   * @returns
   */
  reload(options?: ReloadOptions): Promise<void>

  /**
   * Saves the store to disk at the store's `path`.
   * @returns
   */
  save(): Promise<void>

  /**
   * Listen to changes on a store key.
   * @param key
   * @param cb
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @since 2.0.0
   */
  onKeyChange<T>(
    key: string,
    cb: (value: T | undefined) => void
  ): Promise<UnlistenFn>

  /**
   * Listen to changes on the store.
   * @param cb
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @since 2.0.0
   */
  onChange<T>(
    cb: (key: string, value: T | undefined) => void
  ): Promise<UnlistenFn>

  /**
   * Close the store and cleans up this resource from memory.
   * **You should not call any method on this object anymore and should drop any reference to it.**
   */
  close(): Promise<void>
}

/**
 * Options to change how a store is reloaded from its on-disk state.
 */
export type ReloadOptions = {
  /**
   * To fully match the store with the on-disk state, ignoring defaults
   */
  ignoreDefaults?: boolean
}
