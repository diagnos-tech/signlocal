// Static output for GitHub Pages: every page of every language is a file, scripts and styles are
// external (the pages' Content-Security-Policy has no 'unsafe-inline'), and file names are kept
// as written (download.html stays download.html, test/ stays test/index.html).

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import sitemap from "@astrojs/sitemap";
import { defineConfig } from "astro/config";
import { parse } from "smol-toml";

const here = (path: string) => fileURLToPath(new URL(path, import.meta.url));
const sdk = (path: string) => here(`../sdk/src/${path}`);

const project = parse(readFileSync(here("../project.toml"), "utf8")) as {
  product: { homepage: string };
};
const homepage = new URL(project.product.homepage);

export default defineConfig({
  site: homepage.origin,
  base: "/",
  trailingSlash: "ignore",
  compressHTML: true,
  prefetch: false,
  build: { format: "preserve", inlineStylesheets: "never" },
  devToolbar: { enabled: false },
  integrations: [
    sitemap({
      i18n: {
        defaultLocale: "en",
        locales: {
          en: "en",
          "pt-br": "pt-BR",
          "pt-pt": "pt-PT",
          es: "es",
          fr: "fr",
          it: "it",
          de: "de",
        },
      },
      filter: (page) => !/\/(404|activate|api)(\/|\.html)?$/.test(new URL(page).pathname),
    }),
  ],
  vite: {
    // One stylesheet for the whole site: fewer requests before the first paint, and cached across pages.
    build: { assetsInlineLimit: 0, cssCodeSplit: false },
    resolve: {
      alias: [
        { find: /^@signlocal\/sdk$/, replacement: sdk("index.ts") },
        { find: /^@signlocal\/sdk\/messages$/, replacement: sdk("messages.ts") },
        { find: /^@signlocal\/sdk\/testing$/, replacement: sdk("testing/index.ts") },
      ],
    },
    server: { fs: { allow: [here("..")] } },
  },
});
