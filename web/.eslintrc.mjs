import js from "@eslint/js";
import globals from "globals";

export default [
  {
    ignores: ["**/third-party/**"]
  },
  js.configs.recommended,
  {
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
        bootstrap: "readonly"
      }
    }
  }
];
