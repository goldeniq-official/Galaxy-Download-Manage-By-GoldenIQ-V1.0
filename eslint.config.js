// SPDX-License-Identifier: RPL-1.5
// X-01: no raw HTML from data anywhere (react/no-danger, no eval-like APIs); hooks rules; TS recommended.
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import hooks from "eslint-plugin-react-hooks";

export default tseslint.config(
  { ignores: ["dist", "extension/dist", "node_modules", "src-tauri", "target", "spikes", "docs", "design", "test-results"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    // node test/build scripts
    files: ["**/*.mjs"],
    languageOptions: { globals: { process: "readonly", console: "readonly", Buffer: "readonly", setTimeout: "readonly", setInterval: "readonly", clearInterval: "readonly", URL: "readonly", fetch: "readonly", chrome: "readonly" } },
  },
  {
    files: ["**/*.{ts,tsx}"],
    plugins: { react, "react-hooks": hooks },
    settings: { react: { version: "19" } },
    rules: {
      ...hooks.configs.recommended.rules,
      "react/no-danger": "error",
      "react/no-danger-with-children": "error",
      "react/jsx-no-script-url": "error",
      "react/jsx-no-target-blank": "error",
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-new-func": "error",
      "no-restricted-properties": ["error", { property: "innerHTML" }, { property: "outerHTML" }, { property: "insertAdjacentHTML" }],
      "no-restricted-syntax": ["error", { selector: "CallExpression[callee.property.name='write'][callee.object.name='document']", message: "no document.write" }],
    },
  },
);
