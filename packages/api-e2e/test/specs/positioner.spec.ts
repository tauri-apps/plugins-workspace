// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin, itWm } from '../helpers/index.js'

// Positions are computed from the window's current monitor and outer size, so
// the specs compute the expected physical position the same way and compare
// once the window manager has applied the move. The tray positions need the
// tray icon's rect, which normally comes from a click on the icon; the spec
// hands one in through `handleIconState`, as a JS tray event handler would.
// The moved window is not put back: each spec file gets a fresh app.

interface Geometry {
  monitor: { x: number; y: number; width: number; height: number }
  window: { width: number; height: number }
}

/** Moves the main window and reports where it ended up along with the geometry the position is derived from. */
function moveAndMeasure(position: string, constrained = false) {
  return tauri(
    async (api, position, constrained) => {
      const win = api.window.getCurrentWindow()
      const { Position } = api.positioner
      const value = Position[position as keyof typeof Position]
      await (constrained
        ? api.positioner.moveWindowConstrained(value)
        : api.positioner.moveWindow(value))
      await new Promise((resolve) => setTimeout(resolve, 500))
      const monitor = await api.window.currentMonitor()
      const size = await win.outerSize()
      const outer = await win.outerPosition()
      return {
        monitor: {
          x: monitor!.position.x,
          y: monitor!.position.y,
          width: monitor!.size.width,
          height: monitor!.size.height
        },
        window: { width: size.width, height: size.height },
        position: { x: outer.x, y: outer.y }
      }
    },
    position,
    constrained
  )
}

function center({ monitor, window }: Geometry) {
  return {
    x: monitor.x + Math.trunc(monitor.width / 2) - Math.trunc(window.width / 2),
    y:
      monitor.y + Math.trunc(monitor.height / 2) - Math.trunc(window.height / 2)
  }
}

describePlugin('positioner', { desktopOnly: true }, () => {
  // must run before any tray rect is handed in
  it('tray positions are rejected until the tray icon reported its rect', async () => {
    const error = await tauriError((api) =>
      api.positioner.moveWindow(api.positioner.Position.TrayCenter)
    )
    expect(error).toMatch(/Tray position not set/)
  })

  it('the Position enum matches the plugin', async () => {
    const names = await tauri((api) =>
      Object.keys(api.positioner.Position).filter((key) => isNaN(Number(key)))
    )
    expect(names).toEqual([
      'TopLeft',
      'TopRight',
      'BottomLeft',
      'BottomRight',
      'TopCenter',
      'BottomCenter',
      'LeftCenter',
      'RightCenter',
      'Center',
      'TrayLeft',
      'TrayBottomLeft',
      'TrayRight',
      'TrayBottomRight',
      'TrayCenter',
      'TrayBottomCenter'
    ])
  })

  itWm('moveWindow centers the window on its monitor', async () => {
    const result = await moveAndMeasure('Center')
    expect(result.position).toEqual(center(result))
  })

  itWm(
    'moveWindowConstrained behaves like moveWindow for screen positions',
    async () => {
      const result = await moveAndMeasure('Center', true)
      expect(result.position).toEqual(center(result))
    }
  )

  itWm('tray positions are relative to the reported tray rect', async () => {
    // a tray "icon" in the middle of the window's monitor, far from any edge,
    // so neither the OS-specific flips nor the constraint kick in
    const tray = await tauri(async (api) => {
      const monitor = await api.window.currentMonitor()
      const rect = {
        x: monitor!.position.x + Math.trunc(monitor!.size.width / 2),
        y: monitor!.position.y + Math.trunc(monitor!.size.height / 2),
        width: 20,
        height: 20
      }
      await api.positioner.handleIconState({
        type: 'Click',
        id: 'e2e',
        position: new api.dpi.PhysicalPosition(rect.x, rect.y),
        rect: {
          position: new api.dpi.PhysicalPosition(rect.x, rect.y),
          size: new api.dpi.PhysicalSize(rect.width, rect.height)
        },
        button: 'Left',
        buttonState: 'Down'
      })
      return rect
    })

    const bottomLeft = await moveAndMeasure('TrayBottomLeft')
    expect(bottomLeft.position).toEqual({ x: tray.x, y: tray.y })

    const bottomCenter = await moveAndMeasure('TrayBottomCenter', true)
    expect(bottomCenter.position).toEqual({
      x: tray.x + tray.width / 2 - Math.trunc(bottomCenter.window.width / 2),
      y: tray.y
    })
  })
})
