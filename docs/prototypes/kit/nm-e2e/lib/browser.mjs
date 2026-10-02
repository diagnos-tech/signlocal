// Launches Chromium with the test extension loaded.

import { existsSync } from "node:fs";

import { chromium } from "playwright-core";

/**
 * Opens a persistent context on `userDataDir` (the folder the host manifest
 * was registered in) with the extension at `extensionDir`.
 *
 * The environment is passed explicitly: Chromium hands it on to the native
 * host, and tests rely on variables such as WEBSIGN_PROBE_PIN reaching it.
 */
export async function launchWithExtension({ executablePath, userDataDir, extensionDir, headed }) {
  const exe = executablePath ?? chromium.executablePath();
  if (!existsSync(exe)) {
    throw new Error(
      `Chromium not found at ${exe}. Run \`npx playwright-core install chromium\`, or pass --chromium <path>.`,
    );
  }
  return chromium.launchPersistentContext(userDataDir, {
    executablePath: exe,
    headless: !headed,
    env: { ...process.env },
    // Playwright disables extensions by default; this test needs one.
    ignoreDefaultArgs: ["--disable-extensions"],
    args: [
      `--disable-extensions-except=${extensionDir}`,
      `--load-extension=${extensionDir}`,
    ],
  });
}
