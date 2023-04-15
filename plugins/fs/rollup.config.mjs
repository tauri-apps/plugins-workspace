import { readFileSync } from "fs";
import { builtinModules } from "module";
import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";

const input = "guest-js/index.ts";
const pkg = JSON.parse(
  readFileSync(new URL("./package.json", import.meta.url), "utf8")
);
const external = [/^@tauri-apps\/api/];

export default [
  {
    input,
    external: Object.keys(pkg.dependencies || {})
      .concat(Object.keys(pkg.peerDependencies || {}))
      .concat(builtinModules)
      .concat(external),
    onwarn: (warning) => {
      throw Object.assign(new Error(), warning);
    },
    strictDeprecations: true,
    output: {
      file: pkg.module,
      format: "es",
      sourcemap: true,
    },
    plugins: [typescript({ sourceMap: true })],
  },
  {
    input,
    onwarn: (warning) => {
      throw Object.assign(new Error(), warning);
    },
    strictDeprecations: true,
    output: {
      file: pkg.browser,
      format: "es",
      sourcemap: true,
      entryFileNames: "[name].min.js",
    },
    plugins: [
      resolve(),
      // terser(),
      typescript({ sourceMap: true }),
    ],
  },
];
