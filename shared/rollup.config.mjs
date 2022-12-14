import { builtinModules } from "module";

import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
// import terser from "@rollup/plugin-terser";

/**
 * Create a base rollup config
 * @param {Record<string,any>} pkg Imported package.json
 * @param {string[]} external Imported package.json
 * @returns {import('rollup').RollupOptions}
 */
export function createConfig({ pkg, external = [] }) {
  return [
    {
      input: "index.ts",
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
      input: "index.ts",
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
}
