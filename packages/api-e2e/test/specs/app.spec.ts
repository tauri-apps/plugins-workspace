// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri } from '../helpers/index.js'

// The example app's own commands and events, which its "Communication" view
// drives; they double as a check that the app under test is the right one.

describe('examples/api', () => {
  it('is built from the example config', async () => {
    const info = await tauri(async (api) => ({
      name: await api.app.getName(),
      version: await api.app.getVersion(),
      identifier: await api.app.getIdentifier(),
      label: api.window.getCurrentWindow().label
    }))
    expect(info).toEqual({
      name: 'Tauri API',
      version: '2.0.0',
      identifier: 'com.tauri.api',
      label: 'main'
    })
  })

  it('perform_request returns the backend response', async () => {
    const response = await tauri((api) =>
      api.core.invoke('perform_request', {
        endpoint: 'dummy endpoint arg',
        body: { id: 5, name: 'test' }
      })
    )
    expect(response).toBe('message response')
  })

  it('log_operation accepts an optional payload', async () => {
    await tauri(async (api) => {
      await api.core.invoke('log_operation', { event: 'tauri-click' })
      await api.core.invoke('log_operation', {
        event: 'tauri-click',
        payload: 'from e2e'
      })
      return null
    })
  })

  it('js-event is answered with rust-event', async () => {
    const reply = await tauri(
      (api) =>
        new Promise<{ data: string }>((resolve, reject) => {
          const webview = api.webview.getCurrentWebview()
          webview
            .listen<{ data: string }>('rust-event', (event) =>
              resolve(event.payload)
            )
            .then((unlisten) => {
              webview
                .emit('js-event', 'this is the payload string')
                .catch(reject)
              setTimeout(() => {
                unlisten()
                reject(new Error('no rust-event reply received'))
              }, 5000)
            })
            .catch(reject)
        })
    )
    expect(reply).toEqual({ data: 'something else' })
  })
})
