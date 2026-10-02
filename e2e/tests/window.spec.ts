import { existsSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "@playwright/test";

import { diagnostics } from "../lib/app.ts";
import { env, NEEDS_WINDOW } from "../lib/suite.ts";

// The real windows, per OS (docs/architecture/testing.md §Screenshots): the
// confirmation window saved its states during the other scenarios; the
// diagnostics window is opened here once per tab.
const TABS = ["browsers", "devices", "certificates", "help"];

test.describe("window screenshots", () => {
  test.skip(!env.window, NEEDS_WINDOW);

  for (const tab of TABS) {
    test(`diagnostics ${tab}`, async () => {
      await diagnostics(env, tab);
      for (const theme of ["light", "dark"]) {
        expect(existsSync(join(env.screenshots, `diagnostics-${tab}-${theme}.png`))).toBe(true);
      }
    });
  }
});
