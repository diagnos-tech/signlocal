/**
 * Playwright configuration. Chromium loads the unpacked extension (persistent
 * context); the app is the `e2e` build found through WEBSIGN_E2E_APP. See
 * docs/architecture/testing.md §E2E and lib/environment.ts.
 *
 * With WEBSIGN_E2E_BROWSER_NAME set, the run proves one installed browser:
 * only the cross-browser core (`browsers/`) runs, then the log audit.
 */

import { defineConfig } from "@playwright/test";

const named = (process.env.WEBSIGN_E2E_BROWSER_NAME ?? "") !== "";
const suite = named
  ? { name: "browsers", testMatch: "browsers/**/*.spec.ts" }
  : { name: "scenarios", testMatch: "tests/**/*.spec.ts" };

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
    suite,
    // The log audit reads what every scenario left behind.
    { name: "log audit", testMatch: "audit/**/*.spec.ts", dependencies: [suite.name] },
  ],
});
