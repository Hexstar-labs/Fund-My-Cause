// Native ESLint 9 flat config.
// eslint-config-next v14 is incompatible with ESLint 9 (uses deprecated
// context.getAncestors / getScope APIs and @rushstack/eslint-patch).
// We replicate the essential rules using ESLint-9-compatible plugin versions
// already present in the workspace root.

import tseslint from "../../node_modules/@typescript-eslint/eslint-plugin/dist/index.js";
import tsParser from "../../node_modules/@typescript-eslint/parser/dist/index.js";
import reactHooks from "../../node_modules/eslint-plugin-react-hooks/index.js";
import { fileURLToPath } from "url";
import { dirname, resolve } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));

/** @type {import("eslint").Linter.Config[]} */
const eslintConfig = [
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        project: resolve(__dirname, "tsconfig.json"),
        tsconfigRootDir: __dirname,
      },
    },
    plugins: {
      "@typescript-eslint": tseslint,
      "react-hooks": reactHooks,
    },
    rules: {
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
      "@typescript-eslint/no-explicit-any": "warn",
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      "no-console": ["warn", { allow: ["warn", "error"] }],
    },
  },
];

export default eslintConfig;
