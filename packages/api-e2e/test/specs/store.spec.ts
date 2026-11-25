// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin, scratchDir } from '../helpers/index.js'

// Store paths are relative to `$APPDATA`, which is also inside the example's
// fs scope, so the specs can inspect what the plugin persists.
const dir = scratchDir('store')
const storePath = `${dir}/e2e.json`

describePlugin('store', () => {
  before(async () => {
    await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      if (await api.fs.exists(dir, { baseDir })) {
        await api.fs.remove(dir, { baseDir, recursive: true })
      }
      await api.fs.mkdir(dir, { baseDir, recursive: true })
    }, dir)
  })

  after(async () => {
    await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      if (await api.fs.exists(dir, { baseDir })) {
        await api.fs.remove(dir, { baseDir, recursive: true })
      }
    }, dir)
  })

  it('set, get, has, keys, values, entries, length and delete', async () => {
    const result = await tauri(async (api, path) => {
      const store = await api.store.load(path, { autoSave: false })
      await store.set('string', 'value')
      await store.set('number', 42)
      await store.set('object', { nested: [1, 2, 3] })
      const snapshot = {
        string: await store.get<string>('string'),
        number: await store.get<number>('number'),
        object: await store.get<{ nested: number[] }>('object'),
        // `undefined` is normalized so the result survives serialization
        missing: (await store.get('missing')) ?? null,
        hasString: await store.has('string'),
        hasMissing: await store.has('missing'),
        keys: (await store.keys()).sort(),
        values: await store.values(),
        entries: (await store.entries()).sort(([a], [b]) => a.localeCompare(b)),
        length: await store.length()
      }
      const deleted = await store.delete('number')
      const deletedAgain = await store.delete('number')
      const lengthAfterDelete = await store.length()
      await store.clear()
      const lengthAfterClear = await store.length()
      await store.close()
      return {
        ...snapshot,
        deleted,
        deletedAgain,
        lengthAfterDelete,
        lengthAfterClear
      }
    }, storePath)

    expect(result.string).toBe('value')
    expect(result.number).toBe(42)
    expect(result.object).toEqual({ nested: [1, 2, 3] })
    expect(result.missing).toBeNull()
    expect(result.hasString).toBe(true)
    expect(result.hasMissing).toBe(false)
    expect(result.keys).toEqual(['number', 'object', 'string'])
    expect(result.values).toHaveLength(3)
    expect(result.entries).toEqual([
      ['number', 42],
      ['object', { nested: [1, 2, 3] }],
      ['string', 'value']
    ])
    expect(result.length).toBe(3)
    expect(result.deleted).toBe(true)
    expect(result.deletedAgain).toBe(false)
    expect(result.lengthAfterDelete).toBe(2)
    expect(result.lengthAfterClear).toBe(0)
  })

  it('save persists to disk and load reads it back', async () => {
    const result = await tauri(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      const store = await api.store.load(path, { autoSave: false })
      await store.set('persisted', { answer: 42 })
      await store.save()
      await store.close()

      const onDisk = JSON.parse(
        await api.fs.readTextFile(path, { baseDir })
      ) as Record<string, unknown>

      const reloaded = await api.store.load(path, { autoSave: false })
      const value = await reloaded.get<{ answer: number }>('persisted')
      await reloaded.close()
      return { onDisk, value }
    }, `${dir}/persisted.json`)
    expect(result.onDisk).toEqual({ persisted: { answer: 42 } })
    expect(result.value).toEqual({ answer: 42 })
  })

  it('autoSave writes changes without an explicit save', async () => {
    const onDisk = await tauri(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      const store = await api.store.load(path, { autoSave: 50 })
      await store.set('auto', true)
      // autoSave is debounced; wait for it to flush
      const deadline = Date.now() + 10_000
      while (!(await api.fs.exists(path, { baseDir }))) {
        if (Date.now() > deadline) {
          throw new Error('store was never auto-saved')
        }
        await new Promise((resolve) => setTimeout(resolve, 100))
      }
      const contents = JSON.parse(
        await api.fs.readTextFile(path, { baseDir })
      ) as Record<string, unknown>
      await store.close()
      return contents
    }, `${dir}/autosave.json`)
    expect(onDisk).toEqual({ auto: true })
  })

  it('defaults apply on load and reset restores them', async () => {
    const result = await tauri(async (api, path) => {
      const store = await api.store.load(path, {
        autoSave: false,
        defaults: { theme: 'dark', volume: 5 }
      })
      const initial = {
        theme: await store.get('theme'),
        volume: await store.get('volume')
      }
      await store.set('theme', 'light')
      await store.set('extra', 1)
      await store.reset()
      const afterReset = {
        theme: await store.get('theme'),
        volume: await store.get('volume'),
        extra: (await store.get('extra')) ?? null,
        length: await store.length()
      }
      await store.close()
      return { initial, afterReset }
    }, `${dir}/defaults.json`)
    expect(result.initial).toEqual({ theme: 'dark', volume: 5 })
    expect(result.afterReset).toEqual({
      theme: 'dark',
      volume: 5,
      extra: null,
      length: 2
    })
  })

  it('reload merges the on-disk state, or replaces it with ignoreDefaults', async () => {
    const result = await tauri(async (api, path) => {
      const store = await api.store.load(path, { autoSave: false })
      await store.set('saved', 1)
      await store.save()
      await store.set('saved', 2)
      await store.set('unsaved', true)
      // a plain reload only re-applies what is on disk on top of the cache
      await store.reload()
      const merged = {
        saved: await store.get('saved'),
        unsaved: (await store.get('unsaved')) ?? null
      }
      // ignoreDefaults makes the store match the disk exactly
      await store.set('unsaved', true)
      await store.reload({ ignoreDefaults: true })
      const replaced = {
        saved: await store.get('saved'),
        unsaved: (await store.get('unsaved')) ?? null
      }
      await store.close()
      return { merged, replaced }
    }, `${dir}/reload.json`)
    expect(result.merged).toEqual({ saved: 1, unsaved: true })
    expect(result.replaced).toEqual({ saved: 1, unsaved: null })
  })

  it('getStore returns the already-loaded instance, or null', async () => {
    const result = await tauri(async (api, path) => {
      const before = await api.store.getStore(path)
      const store = await api.store.load(path, { autoSave: false })
      await store.set('shared', 'yes')
      const existing = await api.store.getStore(path)
      const sharedValue = existing ? await existing.get<string>('shared') : null
      await store.close()
      const afterClose = await api.store.getStore(path)
      return { before, sharedValue, afterClose }
    }, `${dir}/get-store.json`)
    expect(result.before).toBeNull()
    expect(result.sharedValue).toBe('yes')
    expect(result.afterClose).toBeNull()
  })

  it('LazyStore initializes on first use and errors after close', async () => {
    const result = await tauri(async (api, path) => {
      const store = new api.store.LazyStore(path, { autoSave: false })
      await store.set('lazy', 'loaded')
      const value = await store.get<string>('lazy')
      await store.close()
      let closedError: string | null = null
      try {
        await store.get('lazy')
      } catch (error) {
        closedError = String(error)
      }
      return { value, closedError }
    }, `${dir}/lazy.json`)
    expect(result.value).toBe('loaded')
    expect(result.closedError).not.toBeNull()
  })

  it('change listeners fire for set and delete', async () => {
    const changes = await tauri(async (api, path) => {
      const store = await api.store.load(path, { autoSave: false })
      const changes: { key: string; value: unknown }[] = []
      const unlisten = await store.onChange((key, value) => {
        // `undefined` does not survive JSON serialization
        changes.push({ key, value: value === undefined ? null : value })
      })
      await store.set('watched', 1)
      await store.delete('watched')
      // events are delivered asynchronously
      await new Promise((resolve) => setTimeout(resolve, 500))
      unlisten()
      await store.close()
      return changes
    }, `${dir}/on-change.json`)
    expect(changes).toEqual([
      { key: 'watched', value: 1 },
      { key: 'watched', value: null }
    ])
  })

  it('onKeyChange only fires for the watched key', async () => {
    const values = await tauri(async (api, path) => {
      const store = await api.store.load(path, { autoSave: false })
      const values: unknown[] = []
      const unlisten = await store.onKeyChange('watched', (value) => {
        values.push(value === undefined ? null : value)
      })
      await store.set('other', 'ignored')
      await store.set('watched', 'a')
      await store.set('watched', 'b')
      await new Promise((resolve) => setTimeout(resolve, 500))
      unlisten()
      await store.close()
      return values
    }, `${dir}/on-key-change.json`)
    expect(values).toEqual(['a', 'b'])
  })
})
