// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import fs from "fs";
import path from "path";
import readline from "readline";

const header = `Copyright 2019-2023 Tauri Programme within The Commons Conservancy
SPDX-License-Identifier: Apache-2.0
SPDX-License-Identifier: MIT`;
const ignoredLicenses = [
  "// Copyright 2021 Flavio Oliveira",
  "// Copyright 2021 Jonas Kruckenberg",
  "// Copyright 2018-2023 the Deno authors.",
];

const extensions = [".rs", ".js", ".ts", ".yml", ".swift", ".kt"];
const ignore = [
  "target",
  "templates",
  "node_modules",
  "gen",
  "dist",
  "dist-js",
  ".svelte-kit",
  "api-iife.js",
  "init-iife.js",
  ".build",
  "notify_rust",
];

async function checkFile(file) {
  if (
    extensions.some((e) => file.endsWith(e)) &&
    !ignore.some((i) => file.includes(`${path.sep}${i}`))
  ) {
    const fileStream = fs.createReadStream(file);
    const rl = readline.createInterface({
      input: fileStream,
      crlfDelay: Infinity,
    });

    let contents = ``;
    let i = 0;
    for await (let line of rl) {
      // ignore empty lines, allow shebang, swift-tools-version and bundler license
      if (
        line.length === 0 ||
        line.startsWith("#!") ||
        line.startsWith("// swift-tools-version:") ||
        ignoredLicenses.includes(line)
      ) {
        continue;
      }

      // strip comment marker
      if (line.startsWith("// ")) {
        line = line.substring(3);
      } else if (line.startsWith("# ")) {
        line = line.substring(2);
      }

      contents += line;
      if (++i === 3) {
        break;
      }
      contents += "\n";
    }
    if (contents !== header) {
      return true;
    }
  }
  return false;
}

async function check(src) {
  const missingHeader = [];

  for (const entry of fs.readdirSync(src, {
    withFileTypes: true,
  })) {
    const p = path.join(src, entry.name);

    if (entry.isSymbolicLink() || ignore.includes(entry.name)) {
      continue;
    }

    if (entry.isDirectory()) {
      const missing = await check(p);
      missingHeader.push(...missing);
    } else {
      const isMissing = await checkFile(p);
      if (isMissing) {
        missingHeader.push(p);
      }
    }
  }

  return missingHeader;
}

const [_bin, _script, ...files] = process.argv;

if (files.length > 0) {
  async function run() {
    const missing = [];
    for (const f of files) {
      const isMissing = await checkFile(f);
      if (isMissing) {
        missing.push(f);
      }
    }
    if (missing.length > 0) {
      console.log(missing.join("\n"));
      process.exit(1);
    }
  }

  run();
} else {
  check(path.resolve(new URL(import.meta.url).pathname, "../../..")).then(
    (missing) => {
      if (missing.length > 0) {
        console.log(missing.join("\n"));
        process.exit(1);
      }
    },
  );
}
