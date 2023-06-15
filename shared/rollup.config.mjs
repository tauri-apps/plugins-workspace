// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { builtinModules } from "module";

import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
import terser from "@rollup/plugin-terser";

/**
 * Create a base rollup config
 * @param {Record<string,any>} pkg Imported package.json
 * @param {string[]} external Imported package.json
 * @returns {import('rollup').RollupOptions}
 */
export function createConfig({ input = "index.ts", pkg, external = [] }) {
  const pluginJsName = pkg.name
    .replace("@tauri-apps/plugin-", "")
    .replace(/-./g, (x) => x[1].toUpperCase());
  const iifeVarName = `__TAURI_${pluginJsName.toUpperCase()}__`;
  return [
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
    {
      input,
      output: {
        file: "src/api-iife.js",
        format: "iife",
        name: iifeVarName,
        // IIFE is in the format `var ${iifeVarName} = (() => {})()`
        // we check if __TAURI__ exists and inject the API object
        banner: "if ('__TAURI__' in window) {",
        // the last `}` closes the if in the banner
        footer: `Object.defineProperty(window.__TAURI__, '${pluginJsName}', { value: ${iifeVarName} }) }`,
      },
      // and var is not guaranteed to assign to the global `window` object so we make sure to assign it
      plugins: [
        resolve(),
        typescript({
          sourceMap: false,
          declaration: false,
          declarationDir: undefined,
        }),
        terser(),
      ],
    },
  ];
}
