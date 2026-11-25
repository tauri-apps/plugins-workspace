// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Lists the plugin crates a change affects, so the Rust workflows only build and test those.
//
// A plugin is affected when:
// - a file in its directory changed (except the JS / docs / Android files `cargo` never reads),
// - a plugin it depends on is affected (e.g. `fs` for `dialog`), or
// - a `Cargo.lock` entry in its dependency tree changed (so a dependency bump only tests the
//   plugins that actually use that dependency).
// Every plugin is affected when a workspace-wide file (root `Cargo.toml`, toolchain, cargo
// config, this script, the files in `GLOBAL_PATHS`) changed, or when there is nothing to diff
// against.
//
// Environment:
// - BASE: the revision to diff `HEAD` against; empty or all zeros means every plugin.
// - ALL: `true` to select every plugin.
// - GLOBAL_PATHS: newline separated paths (usually the calling workflow) that affect every plugin.
// - SHARD_SIZE: the maximum number of plugins per shard (default 8).
//
// Writes `packages` (JSON array) and `shards` (JSON array of `{ name, packages, save }`, where
// `packages` is space separated and `save` is true for the shard that should save the cache)
// to `$GITHUB_OUTPUT`.

import { execFileSync } from 'child_process'
import fs from 'fs'
import path from 'path'

// Crates whose dependency trees cover most of the other plugins' dependencies. They go first so
// the shard that saves the cache (the first one) restores as much as possible for the others.
const preferredFirst = [
  'tauri-plugin-upload',
  'tauri-plugin-sql',
  'tauri-plugin-updater',
  'tauri-plugin-single-instance'
]

const globalPaths = [
  'Cargo.toml',
  'rust-toolchain',
  'rust-toolchain.toml',
  '.cargo/',
  '.scripts/ci/affected-rust-packages.js',
  ...(process.env.GLOBAL_PATHS ?? '')
    .split('\n')
    .map((p) => p.trim())
    .filter(Boolean)
]

// Paths inside a plugin directory that never reach `cargo`.
const ignoredInPlugin = [
  /^guest-js\//,
  /^dist-js\//,
  /^node_modules\//,
  /^android\//,
  /(^|\/)[^/]+\.md$/,
  /^package\.json$/,
  /^tsconfig\.json$/,
  /^rollup\.config\.js$/,
  /^api-iife\.js$/,
  /^banner\.png$/,
  /^LICENSE/,
  /^contributors\//
]

function git(...args) {
  return execFileSync('git', args, {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'pipe']
  })
}

function readPlugins() {
  const plugins = new Map() // directory -> crate name
  for (const dir of fs.readdirSync('plugins')) {
    const manifest = path.join('plugins', dir, 'Cargo.toml')
    if (!fs.existsSync(manifest)) continue
    const name = fs
      .readFileSync(manifest, 'utf8')
      .match(/^\[package\][^[]*?^name\s*=\s*"([^"]+)"/m)?.[1]
    if (name) plugins.set(dir, name)
  }
  return plugins
}

// Parses the `[[package]]` entries of a Cargo.lock into a map of
// `name version (source)` -> `{ name, version, source, checksum, dependencies, fingerprint }`.
function parseLock(text) {
  const packages = new Map()
  for (const block of text.split(/^\[\[package\]\]$/m).slice(1)) {
    const field = (key) =>
      block.match(new RegExp(`^${key} = "([^"]*)"`, 'm'))?.[1]
    const name = field('name')
    const version = field('version')
    const source = field('source')
    const dependencies = [
      ...(block.match(/^dependencies = \[([^\]]*)\]/m)?.[1] ?? '').matchAll(
        /"([^"]+)"/g
      )
    ].map((m) => m[1])
    const id = source ? `${name} ${version} (${source})` : `${name} ${version}`
    const checksum = field('checksum')
    packages.set(id, { name, version, source, checksum, dependencies })
  }

  const byName = new Map()
  for (const [id, pkg] of packages) {
    byName.set(pkg.name, [...(byName.get(pkg.name) ?? []), id])
  }
  // dependency entries are `name`, `name version` or `name version (source)`
  const resolve = (dep) => {
    if (packages.has(dep)) return dep
    const [name, version] = dep.split(' ')
    const candidates = byName.get(name) ?? []
    return version
      ? candidates.find((id) => packages.get(id).version === version)
      : candidates[0]
  }
  for (const pkg of packages.values()) {
    pkg.dependencies = pkg.dependencies.map(resolve).filter(Boolean).sort()
    // what identifies the resolved entry: dependencies are compared by what they resolve to, as
    // cargo spells a dependency differently (`name` or `name version`) depending on how many
    // versions of it the lockfile holds
    pkg.fingerprint = JSON.stringify([pkg.checksum, pkg.dependencies])
  }
  return packages
}

function affectedPackages(plugins) {
  const all = [...plugins.values()]
  const base = process.env.BASE?.trim()

  if (process.env.ALL === 'true') {
    console.log('every plugin selected')
    return all
  }
  if (!base || /^0+$/.test(base)) {
    console.log('nothing to diff against, selecting every plugin')
    return all
  }

  let changedFiles
  try {
    try {
      git('cat-file', '-e', `${base}^{commit}`)
    } catch {
      git('fetch', '--no-tags', '--depth=1', 'origin', base)
    }
    changedFiles = git('diff', '--name-only', base, 'HEAD')
      .split('\n')
      .filter(Boolean)
  } catch (error) {
    console.log(
      `could not diff against ${base}, selecting every plugin: ${error.stderr ?? error}`
    )
    return all
  }
  console.log(`changed files:\n  ${changedFiles.join('\n  ')}`)

  const global = changedFiles.find((file) =>
    globalPaths.some((p) => (p.endsWith('/') ? file.startsWith(p) : file === p))
  )
  if (global) {
    console.log(`${global} affects every plugin`)
    return all
  }

  const lock = parseLock(fs.readFileSync('Cargo.lock', 'utf8'))
  const seeds = new Set()

  for (const file of changedFiles) {
    const [, dir, rest] = file.match(/^plugins\/([^/]+)\/(.+)$/) ?? []
    const name = plugins.get(dir)
    if (!name || ignoredInPlugin.some((pattern) => pattern.test(rest))) continue
    for (const [id, pkg] of lock) {
      if (pkg.name === name && !pkg.source) seeds.add(id)
    }
  }

  if (changedFiles.includes('Cargo.lock')) {
    let baseLock
    try {
      baseLock = parseLock(git('show', `${base}:Cargo.lock`))
    } catch {
      console.log('no Cargo.lock to compare with, selecting every plugin')
      return all
    }
    for (const [id, pkg] of lock) {
      if (baseLock.get(id)?.fingerprint !== pkg.fingerprint) seeds.add(id)
    }
  }

  if (seeds.size) console.log(`changed crates:\n  ${[...seeds].join('\n  ')}`)

  // a plugin is affected when its dependency tree (itself included) reaches a seed,
  // so walk the reverse dependency graph from the seeds
  const dependents = new Map()
  for (const [id, pkg] of lock) {
    for (const dep of pkg.dependencies) {
      dependents.set(dep, [...(dependents.get(dep) ?? []), id])
    }
  }
  const reached = new Set(seeds)
  const queue = [...seeds]
  while (queue.length) {
    for (const dependent of dependents.get(queue.pop()) ?? []) {
      if (!reached.has(dependent)) {
        reached.add(dependent)
        queue.push(dependent)
      }
    }
  }

  const reachedWorkspaceCrates = new Set(
    [...reached]
      .map((id) => lock.get(id))
      .filter((pkg) => !pkg.source)
      .map((pkg) => pkg.name)
  )
  return all.filter((name) => reachedWorkspaceCrates.has(name))
}

const plugins = readPlugins()
const packages = affectedPackages(plugins).sort((a, b) => {
  const rank = (p) =>
    preferredFirst.includes(p) ? preferredFirst.indexOf(p) : Infinity
  return rank(a) - rank(b) || a.localeCompare(b)
})

const shardSize = Number(process.env.SHARD_SIZE) || 8
const shardCount = Math.ceil(packages.length / shardSize)
const perShard = Math.ceil(packages.length / Math.max(shardCount, 1))
const shards = Array.from({ length: shardCount }, (_, i) => {
  const shard = packages.slice(i * perShard, (i + 1) * perShard)
  return {
    name:
      shard.length <= 3
        ? shard.map((p) => p.replace('tauri-plugin-', '')).join(', ')
        : `${i + 1}/${shardCount}`,
    packages: shard.join(' '),
    save: i === 0
  }
})

console.log(`affected packages:\n  ${packages.join('\n  ') || '(none)'}`)

if (process.env.GITHUB_OUTPUT) {
  fs.appendFileSync(
    process.env.GITHUB_OUTPUT,
    `packages=${JSON.stringify(packages)}\nshards=${JSON.stringify(shards)}\n`
  )
}
