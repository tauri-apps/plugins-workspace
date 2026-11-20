// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin } from '../helpers/index.js'

// The driver launches the app without arguments, so the matches reflect the
// CLI definition in the example's `tauri.conf.json` with nothing set. The
// example only registers the plugin on desktop, so the suite is skipped on
// mobile (which has no command line to begin with).

describePlugin('cli', { desktopOnly: true }, () => {
  it('getMatches reports every defined argument as unset', async () => {
    const matches = await tauri((api) => api.cli.getMatches())
    expect(Object.keys(matches.args).sort()).toEqual([
      'config',
      'theme',
      'verbose'
    ])
    // flags resolve to `false`, arguments taking a value to `null`
    expect(matches.args.verbose).toEqual({ value: false, occurrences: 0 })
    expect(matches.args.config).toEqual({ value: null, occurrences: 0 })
    expect(matches.args.theme).toEqual({ value: null, occurrences: 0 })
  })

  it('getMatches reports no subcommand', async () => {
    const subcommand = await tauri(
      async (api) => (await api.cli.getMatches()).subcommand
    )
    expect(subcommand).toBeNull()
  })
})
