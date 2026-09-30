import { expect, test } from "@playwright/test";

import { launch } from "../lib/browser.ts";
import { newMessage, ready, sign, status } from "../lib/fixture.ts";
import { env } from "../lib/suite.ts";

// Scenario 6 (docs/architecture/testing.md §5): what a page learns before
// anything can be signed. `status()` never opens a window.
test.describe("detection", () => {
  test("extension and app installed: ready", async () => {
    const session = await launch(env, { confirm: "wait" });
    try {
      const page = await session.open();
      await ready(page);
      expect(await status(page)).toMatchObject({
        extension: { installed: true },
        app: { installed: true, outdated: false },
        remembered: false,
        ready: true,
      });
    } finally {
      await session.close();
    }
  });

  test("no extension: ExtensionMissing", async () => {
    const session = await launch(env, { confirm: "wait", extension: false });
    try {
      const page = await session.open();
      await ready(page);
      expect(await status(page)).toMatchObject({ extension: { installed: false }, ready: false });
      const reply = await sign(page, { hash: "SHA-256", message: newMessage() });
      expect(reply).toMatchObject({ ok: false, code: "ExtensionMissing" });
    } finally {
      await session.close();
    }
  });

  test("no native messaging host: AppMissing", async () => {
    const session = await launch(env, { confirm: "wait", host: "none" });
    try {
      const page = await session.open();
      await ready(page);
      expect(await status(page)).toMatchObject({
        extension: { installed: true },
        app: { installed: false },
        ready: false,
      });
      const reply = await sign(page, { hash: "SHA-256", message: newMessage() });
      expect(reply).toMatchObject({ ok: false, code: "AppMissing" });
    } finally {
      await session.close();
    }
  });

  test("an old app: AppOutdated", async () => {
    // The fake old app answers `hello` as version 0.0.1 (fixtures/old-app.mjs).
    // The popup's outdated card is not driven here: Playwright does not see
    // the toolbar popup, and the popup only answers its own page. Its state
    // and text for this probe are unit-tested (extension/test/popup-*.test.ts).
    const session = await launch(env, { confirm: "wait", host: "old-app" });
    try {
      const page = await session.open();
      await ready(page);
      expect(await status(page)).toMatchObject({
        extension: { installed: true },
        app: { installed: true, outdated: true },
        ready: false,
      });
      const reply = await sign(page, { hash: "SHA-256", message: newMessage() });
      expect(reply).toMatchObject({ ok: false, code: "AppOutdated" });
    } finally {
      await session.close();
    }
  });
});
