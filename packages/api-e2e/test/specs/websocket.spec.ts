// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'
import {
  WEBSOCKET_FIXTURE_URL,
  WEBSOCKET_CLOSE_REQUEST,
  WEBSOCKET_CLOSE_CODE,
  WEBSOCKET_CLOSE_REASON
} from '../helpers/server.js'

// Every test talks to the fixture server's `/ws` echo endpoint (through
// `adb reverse` on Android). Messages reach the page through the plugin's
// channel, so each page function collects them until it has what it expects,
// with its own timeout so a missing message fails with a readable error.

interface Message {
  type: string
  data: unknown
}

describePlugin('websocket', () => {
  it('echoes text and binary messages', async () => {
    const messages = await tauri(async (api, url) => {
      const ws = await api.websocket.connect(url)
      const received: Message[] = []
      // a copy taken once the echoes are in: the close acknowledgement that
      // `disconnect` triggers may arrive before this function returns
      const echoes = await new Promise<Message[]>((resolve, reject) => {
        setTimeout(() => reject(new Error('echo not received')), 5000)
        ws.addListener((message) => {
          received.push(message)
          if (received.length === 3) resolve(received.slice())
        })
        ws.send('hello')
          .then(() => ws.send([1, 2, 3]))
          .then(() => ws.send({ type: 'Text', data: 'explicit' }))
          .catch(reject)
      })
      await ws.disconnect()
      return echoes
    }, WEBSOCKET_FIXTURE_URL)
    expect(messages).toEqual([
      { type: 'Text', data: 'hello' },
      { type: 'Binary', data: [1, 2, 3] },
      { type: 'Text', data: 'explicit' }
    ])
  })

  it('a ping is answered with a pong carrying its payload', async () => {
    const message = await tauri(async (api, url) => {
      const ws = await api.websocket.connect(url)
      const pong = await new Promise<Message>((resolve, reject) => {
        setTimeout(() => reject(new Error('pong not received')), 5000)
        ws.addListener((message) => {
          if (message.type === 'Pong') resolve(message)
        })
        ws.send({ type: 'Ping', data: [7, 8, 9] }).catch(reject)
      })
      await ws.disconnect()
      return pong
    }, WEBSOCKET_FIXTURE_URL)
    expect(message).toEqual({ type: 'Pong', data: [7, 8, 9] })
  })

  it('sends the configured headers with the upgrade request', async () => {
    const headers = await tauri(async (api, url) => {
      const ws = await api.websocket.connect(url, {
        headers: { 'x-e2e-header': 'from the plugin' }
      })
      const message = await new Promise<Message>((resolve, reject) => {
        setTimeout(() => reject(new Error('headers not received')), 5000)
        ws.addListener(resolve)
      })
      await ws.disconnect()
      return JSON.parse(message.data as string) as Record<string, string>
    }, `${WEBSOCKET_FIXTURE_URL}/headers`)
    expect(headers['x-e2e-header']).toBe('from the plugin')
  })

  it('a listener stops receiving once removed', async () => {
    const counts = await tauri(async (api, url) => {
      const ws = await api.websocket.connect(url)
      let removed = 0
      let kept = 0
      const remove = ws.addListener(() => removed++)
      // counted once both echoes are in, before `disconnect`'s close
      // acknowledgement can reach the listener still attached
      const counts = await new Promise<{ removed: number; kept: number }>(
        (resolve, reject) => {
          setTimeout(() => reject(new Error('echoes not received')), 5000)
          ws.addListener(() => {
            kept++
            if (kept === 1) {
              remove()
              ws.send('second').catch(reject)
            } else {
              resolve({ removed, kept })
            }
          })
          ws.send('first').catch(reject)
        }
      )
      await ws.disconnect()
      return counts
    }, WEBSOCKET_FIXTURE_URL)
    expect(counts).toEqual({ removed: 1, kept: 2 })
  })

  it('a close from the server is delivered and ends the connection', async () => {
    const result = await tauri(
      async (api, url, closeRequest) => {
        const ws = await api.websocket.connect(url)
        const close = await new Promise<Message>((resolve, reject) => {
          setTimeout(() => reject(new Error('close not received')), 5000)
          ws.addListener((message) => {
            if (message.type === 'Close') resolve(message)
          })
          ws.send(closeRequest).catch(reject)
        })
        let sendError = ''
        try {
          await ws.send('after close')
        } catch (error) {
          sendError = String(error)
        }
        return { close, sendError }
      },
      WEBSOCKET_FIXTURE_URL,
      WEBSOCKET_CLOSE_REQUEST
    )
    expect(result.close).toEqual({
      type: 'Close',
      data: { code: WEBSOCKET_CLOSE_CODE, reason: WEBSOCKET_CLOSE_REASON }
    })
    expect(result.sendError).toMatch(/connection not found/)
  })

  it('disconnect sends a normal close frame', async () => {
    const close = await tauri(async (api, url) => {
      const ws = await api.websocket.connect(url)
      // the server acknowledges the close with the same frame
      return await new Promise<Message>((resolve, reject) => {
        setTimeout(() => reject(new Error('close not received')), 5000)
        ws.addListener((message) => {
          if (message.type === 'Close') resolve(message)
        })
        ws.disconnect().catch(reject)
      })
    }, WEBSOCKET_FIXTURE_URL)
    expect(close).toEqual({
      type: 'Close',
      data: { code: 1000, reason: 'Disconnected by client' }
    })
  })

  it('connecting to a closed port is rejected', async () => {
    // port 1 (tcpmux) is never listening on the loopback interface
    const error = await tauriError((api) =>
      api.websocket.connect('ws://127.0.0.1:1/')
    )
    expect(error).toMatch(/refused|connect/i)
  })

  it('invalid URLs and header names are rejected', async () => {
    const url = await tauriError((api) => api.websocket.connect('not a url'))
    expect(url).toMatch(/invalid uri/i)
    const header = await tauriError(
      (api, url) =>
        api.websocket.connect(url, { headers: [['bad header', 'value']] }),
      WEBSOCKET_FIXTURE_URL
    )
    // Chromium (Android) already rejects the name in the `Headers` constructor;
    // WebKit passes it through and the plugin rejects it
    expect(header).toMatch(/invalid (header )?name/i)
  })

  it('sending an unsupported message type throws', async () => {
    const error = await tauriError(async (api, url) => {
      const ws = await api.websocket.connect(url)
      try {
        await ws.send(42 as unknown as string)
      } finally {
        await ws.disconnect()
      }
    }, WEBSOCKET_FIXTURE_URL)
    expect(error).toMatch(/invalid `message` type/)
  })
})
