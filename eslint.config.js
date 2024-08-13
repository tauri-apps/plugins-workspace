// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import eslint from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginSecurity from "eslint-plugin-security";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "**/target",
      "**/node_modules",
      "**/examples",
      "**/dist",
      "**/dist-js",
      "**/build",
      "**/api-iife.js",
      "**/init-iife.js",
      "**/init.js",
      "**/rollup.config.js",
      "**/bindings.ts",
      ".scripts",
      "eslint.config.js",
    ],
  },
  eslint.configs.recommended,
  eslintConfigPrettier,
  eslintPluginSecurity.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: { project: true, tsconfigRootDir: import.meta.dirname },
    },
  },
);
