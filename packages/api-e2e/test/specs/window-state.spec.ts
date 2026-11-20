// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, eventually, describePlugin, itWm } from '../helpers/index.js'

interface WindowOptions {
  width?: number
  height?: number
  x?: number
  y?: number
  titleBarStyle?: 'visible' | 'transparent' | 'overlay'
}

interface Geometry {
  inner: { width: number; height: number }
  outer: { width: number; height: number }
  position: { x: number; y: number }
}

const createWindow = (label: string, options: WindowOptions) =>
  tauri(
    (api, label, options) =>
      new Promise<void>((resolve, reject) => {
        const window = new api.webviewWindow.WebviewWindow(label, options)
        window.once('tauri://created', () => resolve()).catch(reject)
        window
          .once('tauri://error', (event) =>
            reject(
              new Error(`window creation failed: ${String(event.payload)}`)
            )
          )
          .catch(reject)
        setTimeout(() => reject(new Error('window creation timed out')), 8000)
      }),
    label,
    options
  )

const geometry = (label: string) =>
  tauri(async (api, label): Promise<Geometry> => {
    const window = await api.webviewWindow.WebviewWindow.getByLabel(label)
    if (!window) throw new Error(`window ${label} not found`)
    const inner = await window.innerSize()
    const outer = await window.outerSize()
    const position = await window.outerPosition()
    return {
      inner: { width: inner.width, height: inner.height },
      outer: { width: outer.width, height: outer.height },
      position: { x: position.x, y: position.y }
    }
  }, label)

const closeWindow = async (label: string) => {
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

const saveState = (flags: 'SIZE' | 'POSITION') =>
  tauri(
    (api, flags) =>
      api.windowState.saveWindowState(
        flags === 'SIZE'
          ? api.windowState.StateFlags.SIZE
          : api.windowState.StateFlags.POSITION
      ),
    flags
  )

const near = (actual: number, expected: number, tolerance: number) =>
  Math.abs(actual - expected) <= tolerance

/**
 * Creates the window `cycles` times. Each time it asks for a size and
 * position different from the saved state, and the window must come back with
 * the geometry of the first one, so any drift between saving and restoring
 * (a title bar counted twice, a scale applied twice) accumulates and shows.
 */
async function expectStableRoundTrip(
  label: string,
  options: WindowOptions,
  flags: 'SIZE' | 'POSITION',
  cycles = 3
) {
  const scale = await tauri((api) =>
    api.window.getCurrentWindow().scaleFactor()
  )
  const tolerance = Math.ceil(scale) * 2

  await createWindow(label, options)
  // let the window settle before reading what will be saved
  const saved = await eventually(async () => {
    const first = await geometry(label)
    await new Promise((resolve) => setTimeout(resolve, 300))
    const second = await geometry(label)
    if (JSON.stringify(first) !== JSON.stringify(second)) {
      throw new Error('window geometry is still changing')
    }
    return second
  })
  await saveState(flags)
  await closeWindow(label)

  for (let cycle = 1; cycle <= cycles; cycle++) {
    await createWindow(label, {
      ...options,
      width: (options.width ?? 500) + 200,
      height: (options.height ?? 400) + 150,
      x: (options.x ?? 100) + 120,
      y: (options.y ?? 100) + 90
    })
    try {
      await eventually(async () => {
        const actual = await geometry(label)
        const ok =
          flags === 'SIZE'
            ? near(actual.inner.width, saved.inner.width, tolerance)
              && near(actual.inner.height, saved.inner.height, tolerance)
              && near(actual.outer.width, saved.outer.width, tolerance)
              && near(actual.outer.height, saved.outer.height, tolerance)
            : near(actual.position.x, saved.position.x, tolerance)
              && near(actual.position.y, saved.position.y, tolerance)
        if (!ok) {
          throw new Error(
            `cycle ${cycle}: ${JSON.stringify(actual)} does not match the saved ${JSON.stringify(saved)}`
          )
        }
      })
      await saveState(flags)
    } finally {
      await closeWindow(label)
    }
  }
}

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

  // The plugin applies the cached state of a label whenever a window with
  // that label is created, which is what persists state across sessions.
  itWm('a new window is restored to the size it was last saved with', () =>
    expectStableRoundTrip(
      'e2e-window-state-size',
      { width: 500, height: 400 },
      'SIZE'
    )
  )

  // An overlay title bar draws over the content, so the webview is as tall as
  // the window. Restoring must not add or remove the title bar height.
  itWm('an overlay title bar window keeps its size across restores', () =>
    expectStableRoundTrip(
      'e2e-window-state-overlay',
      { width: 500, height: 400, titleBarStyle: 'overlay' },
      'SIZE'
    )
  )

  itWm('a new window is restored to the position it was last saved at', () =>
    expectStableRoundTrip(
      'e2e-window-state-position',
      { width: 500, height: 400, x: 150, y: 120 },
      'POSITION'
    )
  )
})
