// The specs' `test`: when a spec pretends to be another browser (`userAgent`), the page also sees
// that browser's client hints (client-hints.ts), so detection reads the same on Playwright's
// Chromium in CI and on a local Chrome. Every spec imports `test` and `expect` from here.

import { test as base } from "@playwright/test";
import { clientHintsFor, installClientHints } from "./client-hints.ts";

export const test = base.extend<{ clientHints: undefined }>({
  clientHints: [
    async ({ context, userAgent }, use) => {
      if (userAgent) await context.addInitScript(installClientHints, clientHintsFor(userAgent));
      await use(undefined);
    },
    { auto: true },
  ],
});

export { devices, expect } from "@playwright/test";
