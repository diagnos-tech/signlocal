// End-to-end tests run Chromium against the built site (dist/) served by scripts/serve.ts,
// which behaves like GitHub Pages. Build first: `bun run build`. The live-status specs need a
// build made with PUBLIC_SIGNLOCAL_TEST_HOOKS=1 (CI does that); the no-release spec needs one
// made with SIGNLOCAL_SITE_RELEASE=none and skips itself otherwise.
// SIGNLOCAL_CHROME points at a Chrome or Chromium binary when Playwright's own is not installed.

import { defineConfig } from "@playwright/test";

const port = Number(process.env.PORT ?? 4173);
const executablePath = process.env.SIGNLOCAL_CHROME;

export default defineConfig({
  testDir: "tests/e2e",
  outputDir: "test-results",
  timeout: 60_000,
  reporter: process.env.CI ? [["github"], ["list"]] : "list",
  fullyParallel: true,
  ...(process.env.CI ? { workers: 2 } : {}),
  use: {
    baseURL: `http://localhost:${port}`,
    locale: "en-US",
    launchOptions: executablePath ? { executablePath } : {},
  },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
  webServer: {
    command: "bun scripts/serve.ts",
    url: `http://localhost:${port}/`,
    reuseExistingServer: true,
    env: { PORT: String(port) },
  },
});
