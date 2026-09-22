// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  itDesktop
} from '../helpers/index.js'

// The mobile implementation only carries plain text: `write_html`, `write_image`
// and `read_image` answer "Unsupported on this platform" there.
describePlugin('clipboard-manager', () => {
  it('writeText and readText round-trip', async () => {
    const text = 'clipboard text from e2e — ✓'
    expect(
      await tauri(async (api, text) => {
        await api.clipboardManager.writeText(text)
        return api.clipboardManager.readText()
      }, text)
    ).toBe(text)
  })

  it('writeText replaces the previous contents', async () => {
    expect(
      await tauri(async (api) => {
        await api.clipboardManager.writeText('first')
        await api.clipboardManager.writeText('second')
        return api.clipboardManager.readText()
      })
    ).toBe('second')
  })

  itDesktop('writeHtml exposes the alt text as plain text', async () => {
    expect(
      await tauri(async (api) => {
        await api.clipboardManager.writeHtml(
          '<b>bold from e2e</b>',
          'bold from e2e (alt)'
        )
        return api.clipboardManager.readText()
      })
    ).toBe('bold from e2e (alt)')
  })

  itDesktop('writeImage and readImage round-trip pixels', async () => {
    // a 2x2 PNG: red, green / blue, white
    const png = [
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2,
      0, 0, 0, 2, 8, 6, 0, 0, 0, 114, 182, 13, 36, 0, 0, 0, 18, 73, 68, 65, 84,
      120, 218, 99, 248, 207, 192, 240, 31, 12, 129, 52, 24, 0, 0, 73, 200, 9,
      247, 3, 217, 100, 241, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130
    ]
    const rgba = [
      255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255
    ]
    const result = await tauri(async (api, png) => {
      // encoded bytes are decoded by the plugin before hitting the clipboard
      await api.clipboardManager.writeImage(new Uint8Array(png))
      const read = await api.clipboardManager.readImage()
      const size = await read.size()
      const bytes = Array.from(await read.rgba())
      await read.close()
      return { size, bytes }
    }, png)
    expect(result.size).toEqual({ width: 2, height: 2 })
    expect(result.bytes).toEqual(rgba)
  })

  itDesktop(
    'writeImage accepts an Image built with the core image API',
    async () => {
      // `window.__TAURI__.image.Image` and the class the plugin's global script
      // sees must be the same one for `transformImage`'s `instanceof` to hold.
      const rgba = [
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255
      ]
      const result = await tauri(async (api, rgba) => {
        const image = await api.image.Image.new(rgba, 2, 2)
        await api.clipboardManager.writeImage(image)
        const read = await api.clipboardManager.readImage()
        const size = await read.size()
        const bytes = Array.from(await read.rgba())
        await image.close()
        await read.close()
        return { size, bytes }
      }, rgba)
      expect(result.size).toEqual({ width: 2, height: 2 })
      expect(result.bytes).toEqual(rgba)
    }
  )

  itDesktop(
    'an Image read from the clipboard can be written back',
    async () => {
      const result = await tauri(async (api) => {
        const read = await api.clipboardManager.readImage()
        await api.clipboardManager.writeText('replaced by text')
        // `Image` instances are passed by resource id
        await api.clipboardManager.writeImage(read)
        const again = await api.clipboardManager.readImage()
        const size = await again.size()
        await read.close()
        await again.close()
        return size
      })
      expect(result).toEqual({ width: 2, height: 2 })
    }
  )

  itDesktop('writeImage accepts a 1x1 image', async () => {
    // a 1x1 transparent PNG
    const png = [
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1,
      0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84,
      120, 156, 99, 96, 0, 2, 0, 0, 5, 0, 1, 122, 94, 171, 63, 0, 0, 0, 0, 73,
      69, 78, 68, 174, 66, 96, 130
    ]
    const size = await tauri(async (api, png) => {
      await api.clipboardManager.writeImage(new Uint8Array(png))
      const read = await api.clipboardManager.readImage()
      const size = await read.size()
      await read.close()
      return size
    }, png)
    expect(size).toEqual({ width: 1, height: 1 })
  })

  it('readImage rejects when the clipboard holds text', async () => {
    const message = await tauriError(async (api) => {
      await api.clipboardManager.writeText('not an image')
      await api.clipboardManager.readImage()
    })
    expect(message.length).toBeGreaterThan(0)
  })

  it('clear empties the clipboard', async () => {
    // reading an empty clipboard rejects on some platforms and yields an
    // empty string on others
    const text = await tauri(async (api) => {
      await api.clipboardManager.writeText('to be cleared')
      await api.clipboardManager.clear()
      try {
        return await api.clipboardManager.readText()
      } catch {
        return ''
      }
    })
    expect(text).toBe('')
  })
})
