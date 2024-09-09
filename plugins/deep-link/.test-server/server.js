// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import http from 'http'
import fs from 'fs'

const hostname = 'localhost'
const port = 8080

const server = http.createServer(function (req, res) {
  console.log(req.url)
  if (req.url == '/.well-known/apple-app-site-association') {
    const association = fs.readFileSync(
      '.well-known/apple-app-site-association'
    )
    res.writeHead(200, { 'Content-Type': 'application/json' })
    res.end(association)
  } else {
    res.writeHead(404)
    res.end('404 NOT FOUND')
  }
})

server.listen(port, hostname, () => {
  console.log('Server started on port', port)
})
