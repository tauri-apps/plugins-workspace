// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'

// The example spawns an echo server on this port: it replies with the request
// body and the request headers, and sets a `session-token` cookie on requests
// that do not carry one. It is also the only `http://` origin in the example's
// http scope.
const echoServer = 'http://localhost:3003'

describePlugin('http', () => {
  it('fetch performs a GET and exposes status, url and headers', async () => {
    const response = await tauri(async (api, url) => {
      const response = await api.http.fetch(url, {
        headers: { 'x-e2e-header': 'present' }
      })
      return {
        ok: response.ok,
        status: response.status,
        url: response.url,
        // the echo server mirrors the request headers back
        echoedHeader: response.headers.get('x-e2e-header'),
        body: await response.text()
      }
    }, echoServer)
    expect(response.ok).toBe(true)
    expect(response.status).toBe(200)
    expect(response.url).toBe(`${echoServer}/`)
    expect(response.echoedHeader).toBe('present')
    expect(response.body).toBe('')
  })

  it('fetch sends a JSON body and parses the echoed response', async () => {
    const payload = { message: 'hello from e2e', nested: { list: [1, 2, 3] } }
    const response = await tauri(
      async (api, url, payload) => {
        const response = await api.http.fetch(`${url}/json`, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(payload)
        })
        return {
          status: response.status,
          contentType: response.headers.get('content-type'),
          body: (await response.json()) as unknown
        }
      },
      echoServer,
      payload
    )
    expect(response.status).toBe(200)
    expect(response.contentType).toBe('application/json')
    expect(response.body).toEqual(payload)
  })

  it('fetch sends binary bodies and reads them back as bytes', async () => {
    const bytes = [0, 1, 2, 127, 128, 254, 255]
    const echoed = await tauri(
      async (api, url, bytes) => {
        const response = await api.http.fetch(`${url}/bytes`, {
          method: 'PUT',
          body: new Uint8Array(bytes)
        })
        return Array.from(new Uint8Array(await response.arrayBuffer()))
      },
      echoServer,
      bytes
    )
    expect(echoed).toEqual(bytes)
  })

  it('fetch sends multipart form data', async () => {
    const body = await tauri(async (api, url) => {
      const form = new FormData()
      form.append('foo', 'baz')
      form.append('bar', 'qux')
      const response = await api.http.fetch(`${url}/form`, {
        method: 'POST',
        body: form
      })
      return {
        contentType: response.headers.get('content-type'),
        text: await response.text()
      }
    }, echoServer)
    expect(body.contentType).toMatch(/^multipart\/form-data; boundary=/)
    expect(body.text).toContain('name="foo"')
    expect(body.text).toContain('baz')
    expect(body.text).toContain('name="bar"')
    expect(body.text).toContain('qux')
  })

  it('the cookie jar stores and replays cookies across requests', async () => {
    const result = await tauri(async (api, url) => {
      // The jar is persisted in the app data dir, so an earlier run (or the
      // requests above) may already hold the cookie: the first request then
      // replays it instead of being handed a new one.
      const first = await api.http.fetch(`${url}/cookies`)
      await first.text()
      // Either way the jar attaches it to the next request, which the echo
      // server mirrors back as a `cookie` header without setting a new one.
      const second = await api.http.fetch(`${url}/cookies`)
      await second.text()
      return {
        setCookie: first.headers.get('set-cookie'),
        replayedFirst: first.headers.get('cookie'),
        replayed: second.headers.get('cookie'),
        setAgain: second.headers.get('set-cookie')
      }
    }, echoServer)
    if (result.replayedFirst === null) {
      expect(result.setCookie).toMatch(/^session-token=test-value/)
    } else {
      expect(result.replayedFirst).toContain('session-token=test-value')
    }
    expect(result.replayed).toContain('session-token=test-value')
    expect(result.setAgain).toBeNull()
  })

  it('fetch can be aborted', async () => {
    const message = await tauriError(async (api, url) => {
      const controller = new AbortController()
      controller.abort()
      await api.http.fetch(`${url}/aborted`, { signal: controller.signal })
    }, echoServer)
    expect(message).toMatch(/abort|cancel/i)
  })

  it('rejects URLs outside the configured scope', async () => {
    const message = await tauriError((api) =>
      api.http.fetch('http://localhost:3999/not-in-scope')
    )
    expect(message).toMatch(/url not allowed on the configured scope/)
  })

  it('network failures reject', async () => {
    // Every in-scope origin is reachable, so route the request through a proxy
    // nothing listens on to force a connection error.
    const message = await tauriError(
      (api, url) =>
        api.http.fetch(url, { proxy: { all: 'http://127.0.0.1:1' } }),
      echoServer
    )
    expect(message).toMatch(/error sending request/)
  })
})
