/**
 * One dev server for every example. `@websign/sdk` resolves to the sources in
 * this repository so the examples always match the SDK they sit next to; in
 * your project, `bun add @websign/sdk` and drop the aliases.
 */

import { fileURLToPath } from "node:url";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

const sdk = (path: string) => fileURLToPath(new URL(`../../sdk/src/${path}`, import.meta.url));
const page = (path: string) => fileURLToPath(new URL(path, import.meta.url));

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: [
      { find: /^@websign\/sdk$/, replacement: sdk("index.ts") },
      { find: /^@websign\/sdk\/messages$/, replacement: sdk("messages.ts") },
      { find: /^@websign\/sdk\/testing$/, replacement: sdk("testing/index.ts") },
    ],
  },
  server: { fs: { allow: [page("../..")] } },
  build: {
    target: "es2022",
    // pdf-lib and PKI.js make the PAdES page large; only that page loads them.
    chunkSizeWarningLimit: 1024,
    rolldownOptions: {
      input: {
        index: page("index.html"),
        vanilla: page("vanilla/index.html"),
        react: page("react/index.html"),
        vue: page("vue/index.html"),
        pades: page("pades/index.html"),
      },
    },
  },
});
