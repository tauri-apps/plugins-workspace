import { readFileSync } from "fs";

import { createConfig } from "../../shared/rollup.config.mjs";

import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
import terser from "@rollup/plugin-terser";

const config = createConfig({
  input: "guest-js/index.ts",
  pkg: JSON.parse(
    readFileSync(new URL("./package.json", import.meta.url), "utf8"),
  ),
  external: [/^@tauri-apps\/api/],
});

config.push({
  input: "guest-js/init.ts",
  output: {
    file: "src/init-iife.js",
    format: "iife",
  },
  plugins: [
    resolve(),
    typescript({
      sourceMap: false,
      declaration: false,
      declarationDir: undefined,
    }),
    terser(),
  ],
});

export default config;
