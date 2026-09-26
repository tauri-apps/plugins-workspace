// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  itDesktop,
  itOn,
  scratchDir
} from '../helpers/index.js'

// Every path below is relative to `BaseDirectory.AppData`, which the example's
// fs scope allows recursively (`fs:scope-appdata-recursive`).
const dir = scratchDir('fs')

describePlugin('fs', () => {
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

  it('mkdir and exists report the scratch directory', async () => {
    const result = await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      return {
        dir: await api.fs.exists(dir, { baseDir }),
        missing: await api.fs.exists(`${dir}/does-not-exist`, { baseDir })
      }
    }, dir)
    expect(result).toEqual({ dir: true, missing: false })
  })

  it('writeTextFile and readTextFile round-trip utf-8 text', async () => {
    const text = 'Hello from the e2e suite — olá, 世界! 🎉\nsecond line\n'
    const read = await tauri(
      async (api, path, text) => {
        const baseDir = api.fs.BaseDirectory.AppData
        await api.fs.writeTextFile(path, text, { baseDir })
        return api.fs.readTextFile(path, { baseDir })
      },
      `${dir}/text.txt`,
      text
    )
    expect(read).toBe(text)
  })

  it('writeTextFile appends when asked to', async () => {
    const read = await tauri(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      await api.fs.writeTextFile(path, 'first', { baseDir })
      await api.fs.writeTextFile(path, ' second', { baseDir, append: true })
      return api.fs.readTextFile(path, { baseDir })
    }, `${dir}/append.txt`)
    expect(read).toBe('first second')
  })

  it('writeFile and writeTextFile work without options', async () => {
    const result = await tauri(async (api, dir) => {
      const path = await api.path.join(
        await api.path.appDataDir(),
        dir,
        'no-options.txt'
      )
      await api.fs.writeTextFile(path, 'text')
      const text = await api.fs.readTextFile(path)
      await api.fs.writeFile(path, new Uint8Array([98, 121, 116, 101, 115]))
      return { text, bytes: await api.fs.readTextFile(path) }
    }, dir)
    expect(result).toEqual({ text: 'text', bytes: 'bytes' })
  })

  it('writeFile and readFile round-trip binary data', async () => {
    const bytes = [0, 1, 2, 3, 250, 251, 252, 253, 254, 255]
    const read = await tauri(
      async (api, path, bytes) => {
        const baseDir = api.fs.BaseDirectory.AppData
        await api.fs.writeFile(path, new Uint8Array(bytes), { baseDir })
        return Array.from(await api.fs.readFile(path, { baseDir }))
      },
      `${dir}/binary.bin`,
      bytes
    )
    expect(read).toEqual(bytes)
  })

  it('stat, lstat and size describe files and directories', async () => {
    const result = await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      const file = `${dir}/stat.txt`
      await api.fs.writeTextFile(file, '0123456789', { baseDir })
      const fileStat = await api.fs.stat(file, { baseDir })
      const fileLstat = await api.fs.lstat(file, { baseDir })
      const dirStat = await api.fs.stat(dir, { baseDir })
      return {
        file: {
          isFile: fileStat.isFile,
          isDirectory: fileStat.isDirectory,
          isSymlink: fileStat.isSymlink,
          size: fileStat.size,
          hasMtime: fileStat.mtime instanceof Date
        },
        lstatSize: fileLstat.size,
        size: await api.fs.size(file, { baseDir }),
        absoluteSize: await api.fs.size(
          await api.path.join(await api.path.appDataDir(), file)
        ),
        dirSize: await api.fs.size(dir, { baseDir }),
        dir: { isFile: dirStat.isFile, isDirectory: dirStat.isDirectory }
      }
    }, dir)
    expect(result.file).toEqual({
      isFile: true,
      isDirectory: false,
      isSymlink: false,
      size: 10,
      hasMtime: true
    })
    expect(result.lstatSize).toBe(10)
    expect(result.size).toBe(10)
    expect(result.absoluteSize).toBe(10)
    expect(result.dirSize).toBeGreaterThanOrEqual(10)
    expect(result.dir).toEqual({ isFile: false, isDirectory: true })
  })

  it('copyFile, rename and readDir', async () => {
    const result = await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      const sub = `${dir}/tree`
      await api.fs.mkdir(`${sub}/nested`, { baseDir, recursive: true })
      await api.fs.writeTextFile(`${sub}/a.txt`, 'a', { baseDir })
      await api.fs.copyFile(`${sub}/a.txt`, `${sub}/b.txt`, {
        fromPathBaseDir: baseDir,
        toPathBaseDir: baseDir
      })
      await api.fs.rename(`${sub}/b.txt`, `${sub}/c.txt`, {
        oldPathBaseDir: baseDir,
        newPathBaseDir: baseDir
      })
      const entries = await api.fs.readDir(sub, { baseDir })
      return {
        entries: entries
          .map((e) => ({
            name: e.name,
            isFile: e.isFile,
            isDirectory: e.isDirectory
          }))
          .sort((x, y) => x.name.localeCompare(y.name)),
        copied: await api.fs.readTextFile(`${sub}/c.txt`, { baseDir }),
        renamedAway: await api.fs.exists(`${sub}/b.txt`, { baseDir })
      }
    }, dir)
    expect(result.entries).toEqual([
      { name: 'a.txt', isFile: true, isDirectory: false },
      { name: 'c.txt', isFile: true, isDirectory: false },
      { name: 'nested', isFile: false, isDirectory: true }
    ])
    expect(result.copied).toBe('a')
    expect(result.renamedAway).toBe(false)
  })

  it('remove deletes files and (recursively) directories', async () => {
    const result = await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      const sub = `${dir}/to-remove`
      await api.fs.mkdir(`${sub}/nested`, { baseDir, recursive: true })
      await api.fs.writeTextFile(`${sub}/nested/file.txt`, 'x', { baseDir })
      await api.fs.remove(`${sub}/nested/file.txt`, { baseDir })
      const fileGone = !(await api.fs.exists(`${sub}/nested/file.txt`, {
        baseDir
      }))
      await api.fs.writeTextFile(`${sub}/nested/other.txt`, 'x', { baseDir })
      let nonRecursiveError: string | null = null
      try {
        await api.fs.remove(sub, { baseDir })
      } catch (error) {
        nonRecursiveError = String(error)
      }
      await api.fs.remove(sub, { baseDir, recursive: true })
      return {
        fileGone,
        nonRecursiveError,
        dirGone: !(await api.fs.exists(sub, { baseDir }))
      }
    }, dir)
    expect(result.fileGone).toBe(true)
    // a non-empty directory cannot be removed without `recursive`
    expect(result.nonRecursiveError).not.toBeNull()
    expect(result.dirGone).toBe(true)
  })

  it('truncate shortens a file', async () => {
    const result = await tauri(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      await api.fs.writeTextFile(path, '0123456789', { baseDir })
      await api.fs.truncate(path, 4, { baseDir })
      const shortened = await api.fs.readTextFile(path, { baseDir })
      await api.fs.truncate(path, undefined, { baseDir })
      return {
        shortened,
        emptied: await api.fs.readTextFile(path, { baseDir })
      }
    }, `${dir}/truncate.txt`)
    expect(result).toEqual({ shortened: '0123', emptied: '' })
  })

  it('readTextFileLines iterates a file line by line', async () => {
    const lines = await tauri(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      await api.fs.writeTextFile(path, 'one\ntwo\r\nthree', { baseDir })
      const result: string[] = []
      for await (const line of await api.fs.readTextFileLines(path, {
        baseDir
      })) {
        result.push(line)
      }
      return result
    }, `${dir}/lines.txt`)
    expect(lines).toEqual(['one', 'two', 'three'])
  })

  it('readTextFileLines closes the file when a loop exits early', async () => {
    const result = await tauri(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      await api.fs.writeTextFile(path, 'one\ntwo\nthree', { baseDir })
      const lines = await api.fs.readTextFileLines(path, { baseDir })
      // the iterator keeps the id of the open file in `rid`
      const state = lines as unknown as { rid: number | null }
      let rid: number | null = null
      let first: string | null = null
      for await (const line of lines) {
        first = line
        rid = state.rid
        break
      }
      let closeError: string | null = null
      try {
        await api.core.invoke('plugin:resources|close', { rid })
      } catch (error) {
        closeError = String(error)
      }
      // iterating again starts over
      const all: string[] = []
      for await (const line of lines) {
        all.push(line)
      }
      return { first, ridAfter: state.rid, closeError, all }
    }, `${dir}/lines-break.txt`)
    expect(result.first).toBe('one')
    expect(result.ridAfter).toBeNull()
    // the resource was already closed by the iterator
    expect(result.closeError).toMatch(/resource id \d+ is invalid/)
    expect(result.all).toEqual(['one', 'two', 'three'])
  })

  it('readTextFileLines rejects when the file cannot be read', async () => {
    // a directory: opening it fails on Windows, reading it fails elsewhere,
    // which used to yield empty lines forever
    const message = await tauriError(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      await api.fs.mkdir(path, { baseDir, recursive: true })
      const lines = await api.fs.readTextFileLines(path, { baseDir })
      for (let i = 0; i < 1000; i++) {
        const { done } = await lines.next()
        if (done) return
      }
      throw new Error('readTextFileLines kept yielding lines for a directory')
    }, `${dir}/lines-dir`)
    expect(message).not.toMatch(/kept yielding/)
    expect(message).toMatch(/failed to (read line|open file)/)
  })

  it('FileHandle supports write, seek, read, stat and truncate', async () => {
    const result = await tauri(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      const encoder = new TextEncoder()
      const decoder = new TextDecoder()

      const created = await api.fs.create(path, { baseDir })
      const written = await created.write(encoder.encode('hello world'))
      const statAfterWrite = await created.stat()
      await created.close()

      const file = await api.fs.open(path, { baseDir, read: true, write: true })
      // seek past "hello " and read the rest
      const position = await file.seek(6, api.fs.SeekMode.Start)
      const buffer = new Uint8Array(32)
      const read = await file.read(buffer)
      const rest = decoder.decode(buffer.subarray(0, read ?? 0))
      // at EOF, read reports null
      const atEof = await file.read(new Uint8Array(8))
      // relative and end-relative seeks
      const fromEnd = await file.seek(-5, api.fs.SeekMode.End)
      const relative = await file.seek(-1, api.fs.SeekMode.Current)
      await file.truncate(5)
      const statAfterTruncate = await file.stat()
      await file.close()

      return {
        written,
        sizeAfterWrite: statAfterWrite.size,
        position,
        read,
        rest,
        atEof,
        fromEnd,
        relative,
        sizeAfterTruncate: statAfterTruncate.size,
        contents: await api.fs.readTextFile(path, { baseDir })
      }
    }, `${dir}/handle.txt`)
    expect(result).toEqual({
      written: 11,
      sizeAfterWrite: 11,
      position: 6,
      read: 5,
      rest: 'world',
      atEof: null,
      fromEnd: 6,
      relative: 5,
      sizeAfterTruncate: 5,
      contents: 'hello'
    })
  })

  it('a closed FileHandle cannot be used again', async () => {
    const message = await tauriError(async (api, path) => {
      const baseDir = api.fs.BaseDirectory.AppData
      const file = await api.fs.create(path, { baseDir })
      await file.close()
      await file.stat()
    }, `${dir}/closed.txt`)
    expect(message).toMatch(/resource id \d+ is invalid/)
  })

  // `:` is not allowed in Windows file names
  itOn(
    ['linux', 'darwin', 'android', 'ios'],
    'relative paths that look like a URL scheme are paths',
    async () => {
      const result = await tauri(async (api) => {
        const baseDir = api.fs.BaseDirectory.AppData
        const path = 'e2e-notes:2024.txt'
        await api.fs.writeTextFile(path, 'notes', { baseDir })
        try {
          return await api.fs.readTextFile(path, { baseDir })
        } finally {
          await api.fs.remove(path, { baseDir })
        }
      })
      expect(result).toBe('notes')
    }
  )

  it('rejects paths outside the configured scope', async () => {
    // `$HOME` itself is not in the example's fs scope (only the app dirs,
    // `$DOWNLOAD` and `$RESOURCE` are).
    const message = await tauriError(async (api) =>
      api.fs.readTextFile('e2e-forbidden.txt', {
        baseDir: api.fs.BaseDirectory.Home
      })
    )
    expect(message).toMatch(/forbidden path/)
  })

  it('rejects paths escaping the scope through `..`', async () => {
    const message = await tauriError(async (api) =>
      api.fs.writeTextFile('../../e2e-escape.txt', 'nope', {
        baseDir: api.fs.BaseDirectory.AppData
      })
    )
    expect(message).toMatch(/cannot traverse directory|forbidden path/)
  })

  // The watch specs are desktop-only: `fs:allow-watch` is granted in the
  // example's desktop capability only.
  itDesktop(
    'watchImmediate reports changes in a watched directory',
    async () => {
      const result = await tauri(async (api, watched) => {
        const baseDir = api.fs.BaseDirectory.AppData
        await api.fs.mkdir(watched, { baseDir, recursive: true })
        const events: { kind: string; paths: string[] }[] = []
        const unwatch = await api.fs.watchImmediate(
          watched,
          (event) => {
            events.push({
              kind:
                typeof event.type === 'string'
                  ? event.type
                  : Object.keys(event.type)[0],
              paths: event.paths
            })
          },
          { baseDir, recursive: true }
        )
        await api.fs.writeTextFile(`${watched}/touched.txt`, 'watched', {
          baseDir
        })
        // give the notifier a moment to deliver
        const deadline = Date.now() + 10_000
        while (
          !events.some((e) => e.paths.some((p) => p.endsWith('touched.txt')))
        ) {
          if (Date.now() > deadline) {
            throw new Error(
              `no watch event for touched.txt, got ${JSON.stringify(events)}`
            )
          }
          await new Promise((resolve) => setTimeout(resolve, 100))
        }
        unwatch()
        return events
      }, `${dir}/watched`)
      expect(
        result.some((e) => e.paths.some((p) => p.endsWith('touched.txt')))
      ).toBe(true)
    }
  )

  itDesktop('watch debounces and unwatch stops delivery', async () => {
    const result = await tauri(async (api, watched) => {
      const baseDir = api.fs.BaseDirectory.AppData
      await api.fs.mkdir(watched, { baseDir, recursive: true })
      let count = 0
      const unwatch = await api.fs.watch(
        watched,
        () => {
          count++
        },
        { baseDir, delayMs: 200 }
      )
      await api.fs.writeTextFile(`${watched}/debounced.txt`, 'a', {
        baseDir
      })
      const deadline = Date.now() + 10_000
      while (count === 0) {
        if (Date.now() > deadline) {
          throw new Error('no debounced watch event received')
        }
        await new Promise((resolve) => setTimeout(resolve, 100))
      }
      const afterFirst = count
      unwatch()
      await api.fs.writeTextFile(`${watched}/after-unwatch.txt`, 'b', {
        baseDir
      })
      await new Promise((resolve) => setTimeout(resolve, 1000))
      return { afterFirst, afterUnwatch: count }
    }, `${dir}/debounced`)
    expect(result.afterFirst).toBeGreaterThan(0)
    expect(result.afterUnwatch).toBe(result.afterFirst)
  })
})
