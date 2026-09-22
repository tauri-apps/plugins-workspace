// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin, isMobile } from '../helpers/index.js'

// The example registers the plugin with a `Webview` target, so records logged
// from the page (and from Rust) are forwarded back to `attachLogger`.
//
// The plugin's default format differs per platform: `[date][time][target][level]
// message` on desktop, and just `[target] message` on mobile, where the platform
// logger (logcat / os_log) already stamps the time and level.

interface Record {
  level: number
  message: string
}

type Level = 'error' | 'warn' | 'info' | 'debug' | 'trace'

/** Logs `message` at `level` and resolves with the records `attachLogger` saw. */
function logAndCollect(level: Level, message: string) {
  return tauri(
    (api, level, message) =>
      new Promise<Record[]>((resolve, reject) => {
        const records: Record[] = []
        api.log
          .attachLogger((record) => {
            if (record.message.includes(message)) {
              records.push(record)
            }
          })
          .then(async (detach) => {
            // eslint-disable-next-line security/detect-object-injection
            await api.log[level](message)
            setTimeout(() => {
              detach()
              resolve(records)
            }, 1000)
          })
          .catch(reject)
      }),
    level,
    message
  )
}

describePlugin('log', () => {
  it('attachLogger receives records logged from the webview', async () => {
    const records = await logAndCollect('info', 'info record from e2e')
    expect(records).toHaveLength(1)
    expect(records[0].level).toBe(3) // LogLevel.Info
    expect(records[0].message).toMatch(
      isMobile
        ? /\[webview[^\]]*\] info record from e2e$/
        : /\[webview[^\]]*\]\[INFO\] info record from e2e$/
    )
  })

  it('records carry the level they were logged at', async () => {
    const error = await logAndCollect('error', 'error record from e2e')
    const warn = await logAndCollect('warn', 'warn record from e2e')
    expect(error[0].level).toBe(5) // LogLevel.Error
    expect(warn[0].level).toBe(4) // LogLevel.Warn
    if (!isMobile) {
      // the level is only part of the formatted message on desktop
      expect(error[0].message).toContain('[ERROR]')
      expect(warn[0].message).toContain('[WARN]')
    }
  })

  it('records below the configured level are dropped', async () => {
    // the example sets the level filter to Info
    const debug = await logAndCollect('debug', 'debug record from e2e')
    const trace = await logAndCollect('trace', 'trace record from e2e')
    expect(debug).toHaveLength(0)
    expect(trace).toHaveLength(0)
  })

  it('log options are accepted', async () => {
    const records = await tauri(
      (api, needle) =>
        new Promise<Record[]>((resolve, reject) => {
          const records: Record[] = []
          api.log
            .attachLogger((record) => {
              if (record.message.includes(needle)) records.push(record)
            })
            .then(async (detach) => {
              await api.log.info(needle, {
                file: 'e2e.spec.ts',
                line: 42,
                keyValues: { suite: 'plugins-e2e' }
              })
              setTimeout(() => {
                detach()
                resolve(records)
              }, 1000)
            })
            .catch(reject)
        }),
      'record with options from e2e'
    )
    expect(records).toHaveLength(1)
    expect(records[0].level).toBe(3)
  })

  it('detaching the logger stops delivery', async () => {
    const count = await tauri(
      (api, needle) =>
        new Promise<number>((resolve, reject) => {
          let count = 0
          api.log
            .attachLogger((record) => {
              if (record.message.includes(needle)) count++
            })
            .then(async (detach) => {
              await api.log.info(needle)
              await new Promise((r) => setTimeout(r, 500))
              detach()
              await api.log.info(needle)
              await new Promise((r) => setTimeout(r, 1000))
              resolve(count)
            })
            .catch(reject)
        }),
      'detached record from e2e'
    )
    expect(count).toBe(1)
  })

  it('attachConsole forwards records to the console', async () => {
    const forwarded = await tauri(
      (api, needle) =>
        new Promise<string[]>((resolve, reject) => {
          const seen: string[] = []
          // Info records are forwarded to `console.info`
          const original = console.info
          console.info = (...args: unknown[]) => {
            const text = args.map(String).join(' ')
            if (text.includes(needle)) seen.push(text)
            original.apply(console, args)
          }
          api.log
            .attachConsole()
            .then(async (detach) => {
              await api.log.info(needle)
              setTimeout(() => {
                detach()
                console.info = original
                resolve(seen)
              }, 1000)
            })
            .catch(reject)
        }),
      'console record from e2e'
    )
    expect(forwarded.length).toBeGreaterThan(0)
  })

  it('records logged from Rust are forwarded to the webview too', async () => {
    // the example's `log_operation` command logs its arguments at Info
    const message = await tauri(
      (api, needle) =>
        new Promise<string>((resolve, reject) => {
          api.log
            .attachLogger((record) => {
              if (record.message.includes(needle)) resolve(record.message)
            })
            .then(() =>
              api.core.invoke('log_operation', {
                event: 'tauri-click',
                payload: needle
              })
            )
            .catch(reject)
          setTimeout(() => reject(new Error('record not received')), 5000)
        }),
      'rust log from e2e'
    )
    expect(message).toContain(
      isMobile ? '[api_lib::cmd] tauri-click' : '[INFO] tauri-click'
    )
    expect(message).toContain('rust log from e2e')
    expect(message).not.toContain('[webview')
  })
})
