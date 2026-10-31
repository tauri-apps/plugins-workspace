// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { runInNewContext } from 'node:vm'

// Exercise the generated initialization script, with only its DOM/IPC inputs
// substituted. Real event propagation and native permissions need WebView tests.
const script = readFileSync(
  process.env.SHELL_INIT_SCRIPT
    ?? new URL('../../plugins/shell/src/init-iife.js', import.meta.url),
  'utf8'
)

function fixture(readyState = 'complete') {
  let onClick
  let onReady
  const calls = []
  runInNewContext(script, {
    document: {
      readyState,
      querySelector: () => ({
        addEventListener: (name, handler) => {
          assert.equal(name, 'click')
          onClick = handler
        }
      })
    },
    window: {
      __TAURI_INTERNALS__: {
        invoke: (command, args) => {
          calls.push({ command, path: args.path })
          return Promise.resolve()
        }
      },
      addEventListener: (name, handler) => {
        assert.equal(name, 'DOMContentLoaded')
        onReady = handler
      }
    }
  })
  return {
    calls,
    ready() {
      onReady()
    },
    click({
      href = 'https://tauri.app',
      target = '_blank',
      prevented = false,
      nested = false,
      anchor = true
    } = {}) {
      const a = { href, target, matches: () => anchor, parentElement: null }
      const e = {
        target: nested ? { matches: () => false, parentElement: a } : a,
        defaultPrevented: prevented,
        preventDefault() {
          this.defaultPrevented = true
        }
      }
      onClick?.(e)
      return e.defaultPrevented
    }
  }
}

for (const readyState of ['complete', 'interactive', 'loading']) {
  test(`respects cancelled clicks when document is ${readyState}`, () => {
    const f = fixture(readyState)
    if (readyState === 'loading') f.ready()
    assert.equal(f.click({ prevented: true, nested: true }), true)
    assert.deepEqual(f.calls, [])
  })
}

for (const href of [
  'https://tauri.app',
  'http://example.com',
  'mailto:hello@example.com',
  'tel:123'
]) {
  test(`continues opening ${href}`, () => {
    const f = fixture()
    assert.equal(f.click({ href, nested: true }), true)
    assert.deepEqual(f.calls, [{ command: 'plugin:shell|open', path: href }])
  })
}

for (const href of [
  'file:///tmp/example',
  'tauri://localhost',
  'javascript:void(0)',
  ''
]) {
  test(`does not intercept unsupported link ${href}`, () => {
    const f = fixture()
    assert.equal(f.click({ href }), false)
    assert.deepEqual(f.calls, [])
  })
}

for (const target of ['', '_self', '_parent', 'named']) {
  test(`does not intercept target ${target}`, () => {
    const f = fixture()
    assert.equal(f.click({ target }), false)
    assert.deepEqual(f.calls, [])
  })
}

test('does not intercept non-anchor elements', () => {
  const f = fixture()
  assert.equal(f.click({ anchor: false }), false)
  assert.deepEqual(f.calls, [])
})

test('waits for DOMContentLoaded before handling links', () => {
  const f = fixture('loading')
  assert.equal(f.click(), false)
  assert.deepEqual(f.calls, [])
  f.ready()
  assert.equal(f.click(), true)
  assert.equal(f.calls.length, 1)
})
