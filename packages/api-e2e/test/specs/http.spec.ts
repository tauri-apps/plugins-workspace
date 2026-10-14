// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'
import { FIXTURE_SERVER_URL, RAW_HEADER_FILENAME } from '../helpers/server.js'

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

  it('fetch exposes non-ASCII response header bytes as a ByteString', async () => {
    // https://github.com/tauri-apps/plugins-workspace/issues/2898
    const result = await tauri(async (api, url) => {
      const response = await api.http.fetch(`${url}/raw-header`)
      await response.text()
      const value = response.headers.get('content-disposition') ?? ''
      // A ByteString holds one byte per char, like a browser exposes it.
      const bytes = Uint8Array.from(value, (c) => c.charCodeAt(0))
      return new TextDecoder().decode(bytes)
    }, FIXTURE_SERVER_URL)
    expect(result).toBe(`attachment; filename="${RAW_HEADER_FILENAME}"`)
  })

  it('fetch sends ByteString request header values as raw bytes', async () => {
    // Each char is one byte (0x80-0xFF) on the wire, as in a browser. The
    // tail is the UTF-8 encoding of `中`, as when a response header is sent back.
    const value = 'x\u00e4\u00ff\u00e4\u00b8\u00ad'
    const echoed = await tauri(
      async (api, url, value) => {
        const response = await api.http.fetch(`${url}/echo`, {
          headers: { 'x-e2e-raw': value }
        })
        const body = (await response.json()) as {
          headers: Record<string, string>
        }
        return body.headers['x-e2e-raw']
      },
      FIXTURE_SERVER_URL,
      value
    )
    // the fixture server decodes header values as ISO-8859-1
    expect(echoed).toBe(value)
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

  // The example configures `userAgent` and `requestHeaders` rules for
  // `/identity/*` paths in its tauri.conf.json, and an `on_request` hook that
  // handles `/identity/hook/*`. The echo server mirrors the headers it received.
  describe('request identity', () => {
    async function echoedHeaders(
      url: string,
      headers: Record<string, string> = {}
    ): Promise<Record<string, string | null>> {
      return await tauri(
        async (api, url, headers) => {
          const response = await api.http.fetch(url, { headers })
          await response.text()
          const names = [
            'user-agent',
            'origin',
            'x-identity',
            'x-hook-webview-label',
            'x-hook-webview-origin'
          ]
          return Object.fromEntries(
            names.map((name) => [name, response.headers.get(name)])
          )
        },
        url,
        headers
      )
    }

    it('sends the configured user agent and the webview origin by default', async () => {
      const headers = await echoedHeaders(`${echoServer}/plain`)
      expect(headers['user-agent']).toBe('tauri-api-example/1.0')
      expect(headers.origin).toMatch(
        /^(tauri:\/\/localhost|http:\/\/tauri\.localhost)$/
      )
      expect(headers['x-identity']).toBeNull()
    })

    it('keeps the user agent set by the frontend', async () => {
      const headers = await echoedHeaders(`${echoServer}/plain`, {
        'user-agent': 'web/1.0'
      })
      expect(headers['user-agent']).toBe('web/1.0')
    })

    it('applies matching rules without replacing frontend headers', async () => {
      let headers = await echoedHeaders(`${echoServer}/identity/rule`)
      expect(headers.origin).toBe('https://desktop.example.com')
      expect(headers['x-identity']).toBe('config')

      headers = await echoedHeaders(`${echoServer}/identity/rule`, {
        'x-identity': 'web'
      })
      expect(headers['x-identity']).toBe('web')
    })

    it('override rules replace frontend headers, later rules win', async () => {
      const headers = await echoedHeaders(`${echoServer}/identity/override/x`, {
        'user-agent': 'web/1.0',
        'x-identity': 'web'
      })
      expect(headers['user-agent']).toBe('override/1.0')
      expect(headers['x-identity']).toBe('override')
      // set by the first rule, which also matches
      expect(headers.origin).toBe('https://desktop.example.com')
    })

    it('rules can remove the origin', async () => {
      const headers = await echoedHeaders(`${echoServer}/identity/no-origin/x`)
      expect(headers.origin).toBeNull()
      expect(headers['x-identity']).toBe('config')
    })

    it('rules and the hook apply again to each redirect target', async () => {
      // the rule for `/identity/*` matches the redirect, not its target
      let headers = await echoedHeaders(
        `${echoServer}/identity/rule/redirect/plain`
      )
      expect(headers['x-identity']).toBeNull()
      expect(headers.origin).toMatch(
        /^(tauri:\/\/localhost|http:\/\/tauri\.localhost)$/
      )

      // a target the rule matches gets its headers
      headers = await echoedHeaders(
        `${echoServer}/plain/redirect/identity/rule`
      )
      expect(headers['x-identity']).toBe('config')
      expect(headers.origin).toBe('https://desktop.example.com')

      headers = await echoedHeaders(
        `${echoServer}/identity/hook/redirect/plain`
      )
      expect(headers['x-hook-webview-label']).toBeNull()
      expect(headers['x-identity']).toBeNull()
    })

    it('the on_request hook runs after the rules', async () => {
      const headers = await echoedHeaders(`${echoServer}/identity/hook/x`)
      expect(headers.origin).toBe('https://hook.example.com')
      expect(headers['x-identity']).toBe('config')
      expect(headers['x-hook-webview-label']).toBe('main')
      expect(headers['x-hook-webview-origin']).toMatch(
        /^(tauri:\/\/localhost|http:\/\/tauri\.localhost)$/
      )
    })

    it('the on_request hook does not run for URLs outside the scope', async () => {
      const allowed = `${echoServer}/identity/hook/allowed`
      const denied = 'http://localhost:3999/identity/hook/denied'
      await echoedHeaders(allowed)
      const message = await tauriError(
        (api, url) => api.http.fetch(url),
        denied
      )
      expect(message).toMatch(/url not allowed on the configured scope/)

      const urls = await tauri((api) =>
        api.core.invoke<string[]>('http_hook_urls')
      )
      expect(urls).toContain(allowed)
      expect(urls).not.toContain(denied)
    })
  })

  describe('cookie jar shared with Rust', () => {
    it('a native client sends the cookies set on webview requests', async () => {
      const cookie = await tauri(async (api, url) => {
        // gets the cookie set, or replays the one an earlier request stored
        await (await api.http.fetch(`${url}/cookies`)).text()
        return await api.core.invoke<string | null>('http_native_cookie')
      }, echoServer)
      expect(cookie).toContain('session-token=test-value')
    })

    it('clearing the jar from Rust affects webview requests', async () => {
      const result = await tauri(async (api, url) => {
        await api.core.invoke('http_clear_cookies')
        // nothing is sent, so the echo server sets the cookie again
        const cleared = await api.http.fetch(`${url}/cookies`)
        await cleared.text()

        await api.core.invoke('http_clear_cookies')
        // the cookie the echo server sets on this native request lands in the
        // shared jar, so the next webview request sends it
        const nativeAfterClear = await api.core.invoke<string | null>(
          'http_native_cookie'
        )
        const replayed = await api.http.fetch(`${url}/cookies`)
        await replayed.text()
        return {
          sentAfterClear: cleared.headers.get('cookie'),
          setAgain: cleared.headers.get('set-cookie'),
          nativeAfterClear,
          replayed: replayed.headers.get('cookie')
        }
      }, echoServer)
      expect(result.sentAfterClear).toBeNull()
      expect(result.setAgain).toMatch(/^session-token=test-value/)
      expect(result.nativeAfterClear).toBeNull()
      expect(result.replayed).toContain('session-token=test-value')
    })
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
