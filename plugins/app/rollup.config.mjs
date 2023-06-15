import { readFileSync } from "fs";

import { createConfig } from "../../shared/rollup.config.mjs";

export default createConfig({
  input: "guest-js/index.ts",
  pkg: JSON.parse(
    readFileSync(new URL("./package.json", import.meta.url), "utf8")
  ),
  external: [/^@tauri-apps\/api/],
});
