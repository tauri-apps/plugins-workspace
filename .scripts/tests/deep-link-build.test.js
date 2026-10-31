// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('../..', import.meta.url))

// Run the real plugin and its build script on a macOS host. The project files
// stand in for Xcode generation; this does not build or launch an iOS app.
test(
  'deep-link build script tracks the generated iOS project',
  {
    skip: process.platform !== 'darwin',
    timeout: 900_000
  },
  async (t) => {
    const directory = mkdtempSync(join(tmpdir(), 'tauri-deep-link-build-'))
    t.after(() => rmSync(directory, { recursive: true, force: true }))
    const env = { ...process.env }
    delete env.CARGO_BUILD_TARGET
    const configuration = {
      mobile: [
        { scheme: ['wecare'], appLink: false },
        { scheme: ['https'], host: 'example.com', appLink: true }
      ]
    }
    env.TAURI_DEEP_LINK_PLUGIN_CONFIG = JSON.stringify(configuration)
    let project = join(directory, 'apple')
    let name = 'repro'

    function files() {
      const app = join(project, `${name}_iOS`)
      return {
        app,
        info: join(app, 'Info.plist'),
        entitlements: join(app, `${name}_iOS.entitlements`)
      }
    }

    function writeInfo() {
      mkdirSync(files().app, { recursive: true })
      writeFileSync(
        files().info,
        `<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.example.repro</string>
</dict></plist>`
      )
    }

    function writeEntitlements() {
      mkdirSync(files().app, { recursive: true })
      writeFileSync(
        files().entitlements,
        `<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>com.apple.developer.associated-domains</key>
<array><string>webcredentials:example.net</string></array>
</dict></plist>`
      )
    }

    function readPlist(file) {
      const result = spawnSync(
        'plutil',
        ['-convert', 'json', '-o', '-', file],
        {
          encoding: 'utf8'
        }
      )
      assert.equal(result.status, 0, result.stderr)
      return JSON.parse(result.stdout)
    }

    function assertPatched(scheme = 'wecare') {
      const info = readPlist(files().info)
      assert.equal(info.CFBundleIdentifier, 'com.example.repro')
      assert.deepEqual(info.CFBundleURLTypes, [
        {
          CFBundleURLName: `$(PRODUCT_BUNDLE_IDENTIFIER).${scheme}`,
          CFBundleURLSchemes: [scheme]
        }
      ])
      assert.deepEqual(
        readPlist(files().entitlements)[
          'com.apple.developer.associated-domains'
        ],
        ['applinks:example.com', 'webcredentials:example.net']
      )
    }

    function build() {
      const result = spawnSync(
        'cargo',
        ['check', '--locked', '-p', 'tauri-plugin-deep-link', '-vv'],
        {
          cwd: root,
          env,
          encoding: 'utf8',
          timeout: 600_000,
          maxBuffer: 64 * 1024 * 1024
        }
      )
      const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`
      assert.equal(result.error, undefined, String(result.error))
      assert.equal(result.status, 0, output.slice(-6000))
      return output
        .split('\n')
        .some(
          (line) =>
            line.startsWith('[tauri-plugin-deep-link ')
            && line.includes(
              'cargo:rerun-if-env-changed=TAURI_DEEP_LINK_PLUGIN_CONFIG'
            )
        )
    }

    function settle() {
      // A write made by the build script can cause one more invocation. Once
      // serialized, unchanged files must stop invalidating Cargo's fingerprint.
      for (let attempt = 0; attempt < 3; attempt++) {
        if (!build()) return
      }
      assert.fail('unchanged iOS project keeps rerunning the build script')
    }

    env.TAURI_IOS_PROJECT_PATH = project
    env.TAURI_IOS_APP_NAME = name
    writeInfo()
    writeEntitlements()

    await t.test('patches the initial project and settles', () => {
      assert.equal(build(), true)
      assertPatched()
      settle()
      assert.equal(build(), false)
    })

    await t.test(
      'reapplies custom schemes after Info.plist regeneration',
      () => {
        writeInfo()
        build()
        assertPatched()
      }
    )

    await t.test(
      'reapplies associated domains after entitlement regeneration',
      () => {
        writeEntitlements()
        build()
        assertPatched()
      }
    )

    await t.test('reapplies both files after project regeneration', () => {
      rmSync(project, { recursive: true })
      writeInfo()
      writeEntitlements()
      build()
      assertPatched()
      settle()
    })

    for (const [label, remove, restore] of [
      ['Info.plist', () => rmSync(files().info), writeInfo],
      ['entitlements', () => rmSync(files().entitlements), writeEntitlements],
      [
        'app directory',
        () => rmSync(files().app, { recursive: true }),
        () => {
          writeInfo()
          writeEntitlements()
        }
      ],
      [
        'project directory',
        () => rmSync(project, { recursive: true }),
        () => {
          writeInfo()
          writeEntitlements()
        }
      ]
    ]) {
      await t.test(`handles absent and subsequently generated ${label}`, () => {
        remove()
        build()
        settle()
        restore()
        build()
        assertPatched()
        settle()
      })
    }

    await t.test('tracks app name changes without a config change', () => {
      name = 'renamed'
      env.TAURI_IOS_APP_NAME = name
      writeInfo()
      writeEntitlements()
      build()
      assertPatched()
    })

    await t.test('tracks project path changes without a config change', () => {
      project = join(directory, 'another-project')
      env.TAURI_IOS_PROJECT_PATH = project
      writeInfo()
      writeEntitlements()
      build()
      assertPatched()
      settle()
    })

    await t.test(
      'tracks project variables becoming absent and present again',
      () => {
        delete env.TAURI_IOS_PROJECT_PATH
        delete env.TAURI_IOS_APP_NAME
        build()
        settle()
        writeInfo()
        writeEntitlements()
        env.TAURI_IOS_PROJECT_PATH = project
        env.TAURI_IOS_APP_NAME = name
        build()
        assertPatched()
        settle()
      }
    )

    await t.test(
      'does not watch unrelated files once both plist files exist',
      () => {
        writeFileSync(
          join(project, 'unrelated.txt'),
          'unrelated project output'
        )
        assert.equal(build(), false)
      }
    )

    await t.test('continues to track plugin configuration changes', () => {
      configuration.mobile[0].scheme = ['changed']
      env.TAURI_DEEP_LINK_PLUGIN_CONFIG = JSON.stringify(configuration)
      build()
      assertPatched('changed')
      settle()
    })
  }
)
