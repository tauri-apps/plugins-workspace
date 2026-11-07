// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { setImmediate } from 'node:timers/promises'
import { test } from 'node:test'
import { runInNewContext } from 'node:vm'

// Exercise the generated script that Rust injects, with only the IPC transport
// replaced. No webview, native permission prompt or notification is created.
const script = readFileSync(
  new URL('../../plugins/notification/src/init-iife.js', import.meta.url),
  'utf8'
)

function initialize({
  url = 'tauri://localhost',
  windows = false,
  invoke = async () => true
} = {}) {
  const calls = []
  const window = {
    location: new URL(url),
    __TAURI_INTERNALS__: {
      invoke: (command, args) => {
        calls.push({ command, args })
        return invoke(command, args)
      }
    }
  }
  runInNewContext(script.replaceAll('__TEMPLATE_windows__', String(windows)), {
    window
  })
  return { window, calls }
}

for (const url of [
  'about:blank',
  'about:blank#startup',
  'about:blank?startup=1'
]) {
  test(`does not prime permission on ${url}`, async () => {
    const { window, calls } = initialize({ url })
    await setImmediate()
    assert.equal(calls.length, 0)
    assert.equal(window.Notification.permission, 'default')
    assert.equal(typeof window.Notification.requestPermission, 'function')
  })
}

test('still primes permission on about:srcdoc', async () => {
  const { window, calls } = initialize({ url: 'about:srcdoc' })
  await setImmediate()
  assert.equal(calls.length, 1)
  assert.equal(window.Notification.permission, 'granted')
})

for (const [response, permission] of [
  [true, 'granted'],
  [false, 'denied'],
  [null, 'default']
]) {
  test(`primes ${permission} on the real document`, async () => {
    const { window, calls } = initialize({ invoke: async () => response })
    await setImmediate()
    assert.equal(window.Notification.permission, permission)
    assert.equal(calls.length, 1)
    assert.equal(calls[0].command, 'plugin:notification|is_permission_granted')
  })
}

test('primes permission when the real document follows about:blank', async () => {
  const startup = initialize({ url: 'about:blank' })
  const loaded = initialize()
  await setImmediate()
  assert.equal(startup.calls.length, 0)
  assert.equal(loaded.calls.length, 1)
  assert.equal(loaded.window.Notification.permission, 'granted')
})

test('keeps the Windows permission shortcut without IPC', async () => {
  const { window, calls } = initialize({ windows: true })
  await setImmediate()
  assert.equal(window.Notification.permission, 'granted')
  assert.equal(calls.length, 0)
})

test('a failed automatic query is handled and an explicit request can retry', async () => {
  const { window, calls } = initialize({
    invoke: async (command) => {
      if (command === 'plugin:notification|is_permission_granted') {
        throw new Error('IPC unavailable')
      }
      return 'granted'
    }
  })
  // node:test also fails this test if the automatic query rejects unhandled.
  await setImmediate()
  assert.equal(window.Notification.permission, 'default')
  assert.equal(await window.Notification.requestPermission(), 'granted')
  assert.equal(window.Notification.permission, 'granted')
  assert.deepEqual(
    calls.map(({ command }) => command),
    [
      'plugin:notification|is_permission_granted',
      'plugin:notification|request_permission'
    ]
  )
})

test('an explicit request still reports a permission error to its caller', async () => {
  const error = new Error('Permission request failed')
  const { window } = initialize({
    invoke: async (command) => {
      if (command === 'plugin:notification|request_permission') {
        throw error
      }
      return null
    }
  })
  await setImmediate()
  await assert.rejects(window.Notification.requestPermission(), error)
  assert.equal(window.Notification.permission, 'default')
})

for (const [response, permission] of [
  ['granted', 'granted'],
  ['denied', 'denied'],
  ['prompt', 'default'],
  ['prompt-with-rationale', 'default']
]) {
  test(`an explicit ${response} response updates the shim to ${permission}`, async () => {
    const { window } = initialize({
      invoke: async (command) =>
        command === 'plugin:notification|request_permission' ? response : null
    })
    await setImmediate()
    assert.equal(await window.Notification.requestPermission(), response)
    assert.equal(window.Notification.permission, permission)
    assert.throws(() => {
      window.Notification.permission = 'granted'
    }, /Readonly property/)
  })
}
