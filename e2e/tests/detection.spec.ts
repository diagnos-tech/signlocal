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
    const session = await launch(env, { confirm: "wait", app: false });
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

  // TODO(gustavo): AppOutdated needs a fake old host answering `hello` with
  // an old protocol range, registered in place of the app.
  test.fixme("an old app: AppOutdated", async () => {});
});
