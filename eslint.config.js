import stylistic from "@stylistic/eslint-plugin"

// House style, as asked for: no semicolons, no dangling commas, and a space
// before the parenthesis of a function. Everything else is there to catch the
// mistakes that actually bite in a browser — an undeclared global, a variable
// that is written and never read.
export default [
  {
    files: ["www/js/**/*.js", "tools/**/*.mjs"],
    languageOptions: {
      ecmaVersion: 2023,
      sourceType: "module",
      globals: {
        window: "readonly", document: "readonly", navigator: "readonly",
        screen: "readonly", location: "readonly", console: "readonly",
        performance: "readonly", requestAnimationFrame: "readonly",
        cancelAnimationFrame: "readonly", setTimeout: "readonly",
        clearTimeout: "readonly", setInterval: "readonly",
        addEventListener: "readonly", removeEventListener: "readonly",
        innerWidth: "readonly", innerHeight: "readonly", scrollTo: "readonly",
        localStorage: "readonly", fetch: "readonly", WebAssembly: "readonly",
        AudioContext: "readonly", ImageData: "readonly",
        Uint8ClampedArray: "readonly", Uint8Array: "readonly",
        process: "readonly", Buffer: "readonly"
      }
    },
    plugins: { "@stylistic": stylistic },
    rules: {
      "@stylistic/semi": ["error", "never"],
      "@stylistic/comma-dangle": ["error", "never"],
      "@stylistic/space-before-function-paren": ["error", "always"],
      "@stylistic/space-in-parens": ["error", "never"],
      "@stylistic/keyword-spacing": ["error", { before: true, after: true }],
      "@stylistic/space-before-blocks": ["error", "always"],
      "@stylistic/arrow-spacing": "error",
      "@stylistic/comma-spacing": "error",
      "@stylistic/indent": ["error", 2, { SwitchCase: 1 }],
      "@stylistic/quotes": ["error", "double", { avoidEscape: true }],
      "@stylistic/no-trailing-spaces": "error",
      "@stylistic/eol-last": ["error", "always"],
      "no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      "no-undef": "error",
      "prefer-const": "error",
      eqeqeq: ["error", "always"]
    }
  }
]
