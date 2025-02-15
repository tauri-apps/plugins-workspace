// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
}

/**
 * Create a new Store or load the existing store with the path.
 *
 * @example
 * ```typescript
 * import { Store } from '@tauri-apps/api/store';
 * const store = await Store.load('store.json');
 * ```
 *
 * @param path Path to save the store in `app_data_dir`
 * @param options Store configuration options
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
 * import { getStore } from '@tauri-apps/api/store';
 * const store = await getStore('store.json');
 * ```
 *
 * @param path Path of the store.
 */
export async function getStore(path: string): Promise<Store | null> {
  return await Store.get(path)
}

/**
 * A lazy loaded key-value store persisted by the backend layer.
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
   * Note that the options are not applied if someone else already created the store
   * @param path Path to save the store in `app_data_dir`
   * @param options Store configuration options
   */
  constructor(
    private readonly path: string,
    private readonly options?: StoreOptions
  ) {}

  /**
   * Init/load the store if it's not loaded already
   */
  async init(): Promise<void> {
    await this.store
  }

  async set(key: string, value: unknown): Promise<void> {
    return (await this.store).set(key, value)
  }

  async get<T>(key: string): Promise<T | undefined> {
    return (await this.store).get<T>(key)
  }

  async has(key: string): Promise<boolean> {
    return (await this.store).has(key)
  }

  async delete(key: string): Promise<boolean> {
    return (await this.store).delete(key)
  }

  async clear(): Promise<void> {
    await (await this.store).clear()
  }

  async reset(): Promise<void> {
    await (await this.store).reset()
  }

  async keys(): Promise<string[]> {
    return (await this.store).keys()
  }

  async values<T>(): Promise<T[]> {
    return (await this.store).values<T>()
  }

  async entries<T>(): Promise<Array<[key: string, value: T]>> {
    return (await this.store).entries<T>()
  }

  async length(): Promise<number> {
    return (await this.store).length()
  }

  async reload(): Promise<void> {
    await (await this.store).reload()
  }

  async save(): Promise<void> {
    await (await this.store).save()
  }

  async onKeyChange<T>(
    key: string,
    cb: (value: T | undefined) => void
  ): Promise<UnlistenFn> {
    return (await this.store).onKeyChange<T>(key, cb)
  }

  async onChange<T>(
    cb: (key: string, value: T | undefined) => void
  ): Promise<UnlistenFn> {
    return (await this.store).onChange<T>(cb)
  }

  async close(): Promise<void> {
    if (this._store) {
      await (await this._store).close()
    }
  }
}

/**
 * A key-value store persisted by the backend layer.
 */
export class Store extends Resource implements IStore {
  private constructor(rid: number) {
    super(rid)
  }

  /**
   * Create a new Store or load the existing store with the path.
   *
   * @example
   * ```typescript
   * import { Store } from '@tauri-apps/api/store';
   * const store = await Store.load('store.json');
   * ```
   *
   * @param path Path to save the store in `app_data_dir`
   * @param options Store configuration options
   */
  static async load(path: string, options?: StoreOptions): Promise<Store> {
    const rid = await invoke<number>('plugin:store|load', {
      path,
      ...options
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
   * import { Store } from '@tauri-apps/api/store';
   * let store = await Store.get('store.json');
   * if (!store) {
   *   store = await Store.load('store.json');
   * }
   * ```
   *
   * @param path Path of the store.
   */
  static async get(path: string): Promise<Store | null> {
    return await invoke<number | null>('plugin:store|get_store', { path }).then(
      (rid) => (rid ? new Store(rid) : null)
    )
  }

  async set(key: string, value: unknown): Promise<void> {
    await invoke('plugin:store|set', {
      rid: this.rid,
      key,
      value
    })
  }

  async get<T>(key: string): Promise<T | undefined> {
    const [value, exists] = await invoke<[T, boolean]>('plugin:store|get', {
      rid: this.rid,
      key
    })
    return exists ? value : undefined
  }

  async has(key: string): Promise<boolean> {
    return await invoke('plugin:store|has', {
      rid: this.rid,
      key
    })
  }

  async delete(key: string): Promise<boolean> {
    return await invoke('plugin:store|delete', {
      rid: this.rid,
      key
    })
  }

  async clear(): Promise<void> {
    await invoke('plugin:store|clear', { rid: this.rid })
  }

  async reset(): Promise<void> {
    await invoke('plugin:store|reset', { rid: this.rid })
  }

  async keys(): Promise<string[]> {
    return await invoke('plugin:store|keys', { rid: this.rid })
  }

  async values<T>(): Promise<T[]> {
    return await invoke('plugin:store|values', { rid: this.rid })
  }

  async entries<T>(): Promise<Array<[key: string, value: T]>> {
    return await invoke('plugin:store|entries', { rid: this.rid })
  }

  async length(): Promise<number> {
    return await invoke('plugin:store|length', { rid: this.rid })
  }

  async reload(): Promise<void> {
    await invoke('plugin:store|reload', { rid: this.rid })
  }

  async save(): Promise<void> {
    await invoke('plugin:store|save', { rid: this.rid })
  }

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
   * Note: This method does not emit change events.
   * @returns
   */
  reload(): Promise<void>

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
