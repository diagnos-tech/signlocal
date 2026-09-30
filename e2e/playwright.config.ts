/**
 * Playwright configuration. Chromium loads the unpacked extension (persistent
 * context); the app is the `e2e` build found through WEBSIGN_E2E_APP. See
 * docs/architecture/testing.md §E2E.
 */

import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests",
  timeout: 120_000,
  retries: 0,
  workers: 1,
  outputDir: "../target/e2e-results",
  use: { trace: "retain-on-failure" },
});
