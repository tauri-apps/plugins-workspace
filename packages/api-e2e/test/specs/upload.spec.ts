// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  scratchDir
} from '../helpers/index.js'
import { FIXTURE_SERVER_URL, DOWNLOAD_FIXTURE_BODY } from '../helpers/server.js'

// The plugin only takes absolute paths; these are resolved against `$APPDATA`
// inside the page, which is inside the example's fs scope so the specs can
// prepare and inspect the files.
const dir = scratchDir('upload')

interface Progress {
  progress: number
  progressTotal: number
  total: number
  transferSpeed: number
}

describePlugin('upload', () => {
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

  it('download writes the response to disk and reports progress', async () => {
    const result = await tauri(
      async (api, url, relativePath) => {
        const baseDir = api.fs.BaseDirectory.AppData
        const path = await api.path.join(
          await api.path.appDataDir(),
          relativePath
        )
        const events: Progress[] = []
        await api.upload.download(
          url,
          path,
          (progress) => events.push(progress),
          new Map([['x-e2e-download', 'yes']])
        )
        return {
          contents: await api.fs.readTextFile(relativePath, { baseDir }),
          events
        }
      },
      `${FIXTURE_SERVER_URL}/download`,
      `${dir}/downloaded.txt`
    )
    expect(result.contents).toBe(DOWNLOAD_FIXTURE_BODY)
    expect(result.events.length).toBeGreaterThan(0)
    const last = result.events[result.events.length - 1]
    const expectedSize = Buffer.byteLength(DOWNLOAD_FIXTURE_BODY)
    // the fixture server sends a Content-Length, so the total is known
    expect(last.total).toBe(expectedSize)
    expect(last.progressTotal).toBe(expectedSize)
    expect(result.events.reduce((sum, event) => sum + event.progress, 0)).toBe(
      expectedSize
    )
  })

  it('download rejects on a non-success status', async () => {
    const message = await tauriError(
      async (api, url, relativePath) =>
        api.upload.download(
          url,
          await api.path.join(await api.path.appDataDir(), relativePath)
        ),
      `${FIXTURE_SERVER_URL}/does-not-exist`,
      `${dir}/missing.txt`
    )
    expect(message).toMatch(/404/)
  })

  it('upload streams a file with the requested method and headers', async () => {
    const contents = 'upload me\n'.repeat(1000)
    const result = await tauri(
      async (api, url, relativePath, contents) => {
        const baseDir = api.fs.BaseDirectory.AppData
        await api.fs.writeTextFile(relativePath, contents, { baseDir })
        const path = await api.path.join(
          await api.path.appDataDir(),
          relativePath
        )
        const events: Progress[] = []
        const response = await api.upload.upload(
          url,
          path,
          (progress) => events.push(progress),
          new Map([['x-e2e-upload', 'yes']]),
          api.upload.HttpMethod.Put
        )
        return { response: JSON.parse(response) as unknown, events }
      },
      `${FIXTURE_SERVER_URL}/echo`,
      `${dir}/to-upload.txt`,
      contents
    )
    const echoed = result.response as {
      method: string
      headers: Record<string, string>
      body: string
    }
    expect(echoed.method).toBe('PUT')
    expect(echoed.headers['x-e2e-upload']).toBe('yes')
    expect(echoed.body).toBe(contents)
    const size = Buffer.byteLength(contents)
    expect(result.events.length).toBeGreaterThan(0)
    expect(result.events[result.events.length - 1].total).toBe(size)
    expect(result.events[result.events.length - 1].progressTotal).toBe(size)
  })

  it('upload defaults to POST', async () => {
    const method = await tauri(
      async (api, url, relativePath) => {
        const baseDir = api.fs.BaseDirectory.AppData
        await api.fs.writeTextFile(relativePath, 'post me', { baseDir })
        const response = await api.upload.upload(
          url,
          await api.path.join(await api.path.appDataDir(), relativePath)
        )
        return (JSON.parse(response) as { method: string }).method
      },
      `${FIXTURE_SERVER_URL}/echo`,
      `${dir}/to-post.txt`
    )
    expect(method).toBe('POST')
  })

  it('upload rejects when the file does not exist', async () => {
    const message = await tauriError(
      async (api, url, relativePath) =>
        api.upload.upload(
          url,
          await api.path.join(await api.path.appDataDir(), relativePath)
        ),
      `${FIXTURE_SERVER_URL}/echo`,
      `${dir}/does-not-exist.txt`
    )
    expect(message).toMatch(/os error 2/)
  })
})
