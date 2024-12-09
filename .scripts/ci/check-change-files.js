#!/usr/bin/env node

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { readFileSync, readdirSync } from 'fs'
import { join } from 'path'

/* const ignorePackages = [
  'api-example',
  'api-example-js',
  'deep-link-example',
  'deep-link-example-js'
] */

const rsOnly = ['localhost', 'persisted-scope', 'single-instance']

function checkChangeFiles(changeFiles) {
  let code = 0

  for (const file of changeFiles) {
    const content = readFileSync(file, 'utf8')
    const [frontMatter] = /^---[\s\S.]*---\n/i.exec(content)
    const packages = frontMatter
      .split('\n')
      .filter((l) => !(l === '---' || !l))
      .map((l) => l.replace(/('|")/g, '').split(':'))

    const rsPackages = Object.fromEntries(
      packages
        .filter((v) => !v[0].endsWith('-js'))
        .map((v) => [v[0], v[1].trim()])
    )
    const jsPackages = Object.fromEntries(
      packages
        .filter((v) => v[0].endsWith('-js'))
        .map((v) => [v[0].slice(0, -3), v[1].trim()])
    )

    for (const pkg in rsPackages) {
      if (rsOnly.includes(pkg)) continue

      if (!jsPackages[pkg]) {
        console.error(
          `Missing "${rsPackages[pkg]}" bump for JS package "${pkg}-js" in ${file}.`
        )
        code = 1
      } else if (rsPackages[pkg] != jsPackages[pkg]) {
        console.error(
          `"${pkg}" and "${pkg}-js" have different version bumps in ${file}.`
        )
        code = 1
      }
    }

    for (const pkg in jsPackages) {
      if (!rsPackages[pkg]) {
        console.error(
          `Missing "${jsPackages[pkg]}" bump for Rust package "${pkg}" in ${file}.`
        )
        code = 1
      } else if (rsPackages[pkg] != jsPackages[pkg]) {
        console.error(
          `"${pkg}" and "${pkg}-js" have different version bumps in ${file}.`
        )
        code = 1
      }
    }
  }

  process.exit(code)
}

const [_bin, _script, ...files] = process.argv

if (files.length > 0) {
  checkChangeFiles(
    files.filter((f) => f.toLowerCase() !== '.changes/readme.md')
  )
} else {
  const changeFiles = readdirSync('.changes')
    .filter((f) => f.endsWith('.md') && f.toLowerCase() !== 'readme.md')
    .map((p) => join('.changes', p))
  checkChangeFiles(changeFiles)
}
