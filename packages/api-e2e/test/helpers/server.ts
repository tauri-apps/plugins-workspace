// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import http from 'node:http'

/**
 * Where the fixture server listens. The port is fixed because the updater
 * endpoint is baked into the app at build time (see `tauri.e2e.conf.json`).
 */
export const FIXTURE_SERVER_PORT = 3004
export const FIXTURE_SERVER_URL = `http://127.0.0.1:${FIXTURE_SERVER_PORT}`

/** Version the `/updater` endpoint advertises by default. */
export const UPDATER_FIXTURE_VERSION = '2.1.0'
export const UPDATER_FIXTURE_NOTES = 'Test update from the e2e fixture server'
/** `{{target}}` values with special behavior on the `/updater` endpoint. */
export const UPDATER_TARGET_NO_UPDATE = 'e2e-no-update'
export const UPDATER_TARGET_OLDER = 'e2e-older'
export const UPDATER_FIXTURE_OLDER_VERSION = '1.0.0'

/** Body served by `GET /download`. */
export const DOWNLOAD_FIXTURE_BODY =
  'hello from the plugins e2e fixture server\n'.repeat(64)

export interface FixtureServer {
  close(): void
}

/**
 * A tiny HTTP server the network-facing specs (updater, upload) talk to:
 *
 * - `GET /updater/{{target}}/{{arch}}/{{current_version}}` — an updater
 *   manifest in the dynamic format. Advertises {@link UPDATER_FIXTURE_VERSION},
 *   or {@link UPDATER_FIXTURE_OLDER_VERSION} when the target is
 *   {@link UPDATER_TARGET_OLDER}, and replies `204 No Content` when it is
 *   {@link UPDATER_TARGET_NO_UPDATE}.
 * - `GET /download` — {@link DOWNLOAD_FIXTURE_BODY} with a `Content-Length`.
 * - `* /echo` — a JSON description of the request (`method`, `url`, `headers`
 *   and the utf-8 `body`).
 */
export function startFixtureServer(): Promise<FixtureServer> {
  const server = http.createServer((req, res) => {
    const chunks: Buffer[] = []
    req.on('data', (chunk: Buffer) => chunks.push(chunk))
    req.on('end', () => {
      const body = Buffer.concat(chunks)
      const url = new URL(req.url ?? '/', FIXTURE_SERVER_URL)
      const [, route, ...rest] = url.pathname.split('/')

      if (route === 'updater' && req.method === 'GET') {
        const [target] = rest
        if (target === UPDATER_TARGET_NO_UPDATE) {
          res.writeHead(204).end()
          return
        }
        json(res, {
          version:
            target === UPDATER_TARGET_OLDER
              ? UPDATER_FIXTURE_OLDER_VERSION
              : UPDATER_FIXTURE_VERSION,
          notes: UPDATER_FIXTURE_NOTES,
          pub_date: '2026-03-01T14:04:20Z',
          url: `${FIXTURE_SERVER_URL}/download`,
          signature: ''
        })
        return
      }

      if (route === 'download' && req.method === 'GET') {
        res
          .writeHead(200, {
            'content-type': 'text/plain',
            'content-length': Buffer.byteLength(DOWNLOAD_FIXTURE_BODY)
          })
          .end(DOWNLOAD_FIXTURE_BODY)
        return
      }

      if (route === 'echo') {
        json(res, {
          method: req.method,
          url: req.url,
          headers: req.headers,
          body: body.toString('utf8')
        })
        return
      }

      res.writeHead(404).end()
    })
  })

  return new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(FIXTURE_SERVER_PORT, '127.0.0.1', () => {
      server.off('error', reject)
      resolve({
        close: () => {
          server.closeAllConnections()
          server.close()
        }
      })
    })
  })
}

function json(res: http.ServerResponse, value: unknown) {
  const payload = JSON.stringify(value)
  res
    .writeHead(200, {
      'content-type': 'application/json',
      'content-length': Buffer.byteLength(payload)
    })
    .end(payload)
}
