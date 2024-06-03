import eslint from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "**/target",
      "**/node_modules",
      "**/dist",
      "**/dist-js",
      "**/rollup.config.mjs",
      "**/vite.config.ts",
      ".scripts",
      "eslint.config.js",
    ],
  },
  eslint.configs.recommended,
  eslintConfigPrettier,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: { project: true, tsconfigRootDir: import.meta.dirname },
    },
  },
);
