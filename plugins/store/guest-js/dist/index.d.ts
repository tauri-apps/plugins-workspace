import { UnlistenFn } from "@tauri-apps/api/event";
/**
 * A key-value store persisted by the backend layer.
 */
export declare class Store {
    path: string;
    constructor(path: string);
    /**
     * Inserts a key-value pair into the store.
     *
     * @param key
     * @param value
     * @returns
     */
    set(key: string, value: unknown): Promise<void>;
    /**
     * Returns the value for the given `key` or `null` the key does not exist.
     *
     * @param key
     * @returns
     */
    get<T>(key: string): Promise<T | null>;
    /**
     * Returns `true` if the given `key` exists in the store.
     *
     * @param key
     * @returns
     */
    has(key: string): Promise<boolean>;
    /**
     * Removes a key-value pair from the store.
     *
     * @param key
     * @returns
     */
    delete(key: string): Promise<boolean>;
    /**
     * Clears the store, removing all key-value pairs.
     *
     * Note: To clear the storage and reset it to it's `default` value, use `reset` instead.
     * @returns
     */
    clear(): Promise<void>;
    /**
     * Resets the store to it's `default` value.
     *
     * If no default value has been set, this method behaves identical to `clear`.
     * @returns
     */
    reset(): Promise<void>;
    /**
     * Returns a list of all key in the store.
     *
     * @returns
     */
    keys(): Promise<string[]>;
    /**
     * Returns a list of all values in the store.
     *
     * @returns
     */
    values(): Promise<string[]>;
    /**
     * Returns a list of all entries in the store.
     *
     * @returns
     */
    entries<T>(): Promise<Array<[key: string, value: T]>>;
    /**
     * Returns the number of key-value pairs in the store.
     *
     * @returns
     */
    length(): Promise<string[]>;
    /**
     * Attempts to load the on-disk state at the stores `path` into memory.
     *
     * This method is useful if the on-disk state was edited by the user and you want to synchronize the changes.
     *
     * Note: This method does not emit change events.
     * @returns
     */
    load(): Promise<void>;
    /**
     * Saves the store to disk at the stores `path`.
     *
     * As the store is only persistet to disk before the apps exit, changes might be lost in a crash.
     * This method let's you persist the store to disk whenever you deem necessary.
     * @returns
     */
    save(): Promise<void>;
    /**
     * Listen to changes on a store key.
     * @param key
     * @param cb
     * @returns A promise resolving to a function to unlisten to the event.
     */
    onKeyChange<T>(key: string, cb: (value: T | null) => void): Promise<UnlistenFn>;
    /**
     * Listen to changes on the store.
     * @param cb
     * @returns A promise resolving to a function to unlisten to the event.
     */
    onChange(cb: (key: string, value: unknown) => void): Promise<UnlistenFn>;
}
