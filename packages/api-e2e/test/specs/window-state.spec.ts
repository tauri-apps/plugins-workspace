// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, eventually, describePlugin, itWm } from '../helpers/index.js'

// The plugin is desktop-only: a mobile window is the whole screen and has no
// state to persist, so the whole suite is skipped there.
describePlugin('window-state', { desktopOnly: true }, () => {
  it('filename reports the state file name', async () => {
    expect(await tauri((api) => api.windowState.filename())).toBe(
      '.window-state.json'
    )
  })

  it('saveWindowState and restoreState resolve', async () => {
    await tauri(async (api) => {
      await api.windowState.saveWindowState(api.windowState.StateFlags.ALL)
      await api.windowState.restoreState('main', api.windowState.StateFlags.ALL)
      await api.windowState.restoreStateCurrent()
      return null
    })
  })

  it('StateFlags combine as a bit set', async () => {
    const flags = await tauri((api) => api.windowState.StateFlags)
    expect(flags.ALL).toBe(
      flags.SIZE
        | flags.POSITION
        | flags.MAXIMIZED
        | flags.VISIBLE
        | flags.DECORATIONS
        | flags.FULLSCREEN
    )
  })

  itWm(
    'a new window is restored to the size it was last saved with',
    async () => {
      // The plugin applies the cached state of a label whenever a window with
      // that label is created, which is what persists sizes across sessions.
      const label = 'e2e-window-state'
      const scale = await tauri((api) =>
        api.window.getCurrentWindow().scaleFactor()
      )

      const create = (width: number, height: number) =>
        tauri(
          (api, label, width, height) =>
            new Promise<void>((resolve, reject) => {
              const window = new api.webviewWindow.WebviewWindow(label, {
                width,
                height
              })
              window.once('tauri://created', () => resolve()).catch(reject)
              window
                .once('tauri://error', (event) =>
                  reject(
                    new Error(
                      `window creation failed: ${String(event.payload)}`
                    )
                  )
                )
                .catch(reject)
              setTimeout(
                () => reject(new Error('window creation timed out')),
                8000
              )
            }),
          label,
          width,
          height
        )
      const innerSize = () =>
        tauri(async (api, label) => {
          const window = await api.webviewWindow.WebviewWindow.getByLabel(label)
          if (!window) throw new Error(`window ${label} not found`)
          const size = await window.innerSize()
          return { width: size.width, height: size.height }
        }, label)
      const close = async () => {
        await tauri(async (api, label) => {
          const window = await api.webviewWindow.WebviewWindow.getByLabel(label)
          await window?.close()
          return null
        }, label)
        await eventually(async () => {
          const labels = await tauri(async (api) =>
            (await api.webviewWindow.getAllWebviewWindows()).map((w) => w.label)
          )
          if (labels.includes(label)) {
            throw new Error('window is still present after close')
          }
        })
      }
      const expectSize = (width: number, height: number) =>
        eventually(async () => {
          const size = await innerSize()
          const tolerance = Math.ceil(scale) * 8
          if (
            Math.abs(size.width - width) > tolerance
            || Math.abs(size.height - height) > tolerance
          ) {
            throw new Error(
              `size ${size.width}x${size.height} not near ${width}x${height}`
            )
          }
        })

      // create the window at one size and save that state
      await create(500, 400)
      await expectSize(500 * scale, 400 * scale)
      const saved = await innerSize()
      await tauri((api) =>
        api.windowState.saveWindowState(api.windowState.StateFlags.SIZE)
      )
      await close()

      // a new window with the same label asks for another size, and gets the
      // saved one back
      await create(700, 600)
      try {
        await expectSize(saved.width, saved.height)
      } finally {
        await close()
      }
    }
  )
})
