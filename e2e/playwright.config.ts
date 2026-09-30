/**
 * Playwright configuration. Chromium loads the unpacked extension (persistent
 * context); the app is the `e2e` build found through WEBSIGN_E2E_APP. See
 * docs/architecture/testing.md §E2E and lib/environment.ts.
 */

import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  globalSetup: "./lib/global-setup.ts",
  timeout: 120_000,
  expect: { timeout: 30_000 },
  retries: 0,
  // One app window at a time: the scenarios share the display and the
  // OS key store.
  workers: 1,
  fullyParallel: false,
  outputDir: "../target/e2e-results",
  use: { trace: "retain-on-failure" },
  projects: [
    { name: "scenarios", testMatch: "tests/**/*.spec.ts" },
    // The log audit reads what every scenario left behind.
    { name: "log audit", testMatch: "audit/**/*.spec.ts", dependencies: ["scenarios"] },
  ],
});
