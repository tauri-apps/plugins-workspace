// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import http from "http";
import fs from "fs";
import path from "path";
import * as url from "url";
const __dirname = url.fileURLToPath(new URL(".", import.meta.url));

const port = 8125;

http
  .createServer(function (request, response) {
    if (request.url === "/.well-known/apple-app-site-association") {
      // eslint-disable-next-line
      fs.readFile(
        path.resolve(__dirname, "apple-app-site-association"),
        function (_error, content) {
          response.writeHead(200);
          response.end(content, "utf-8");
        },
      );
    } else {
      response.writeHead(404);
      response.end();
    }
  })
  .listen(port);

console.log(`Server running at http://127.0.0.1:${port}/`);
