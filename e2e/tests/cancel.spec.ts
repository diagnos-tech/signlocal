import { expect, type Page, test } from "@playwright/test";

import { launch } from "../lib/browser.ts";
import { abort, newMessage, prepareCalled, ready, result, sign, start } from "../lib/fixture.ts";
import { startServer } from "../lib/server.ts";
import { env, expectSigned, keyOf, NEEDS_WINDOW } from "../lib/suite.ts";
import { forget, waitForState } from "../lib/window.ts";

// Scenario 5 (docs/architecture/testing.md §5): the person cancels in the
// window, or the page gives up. Every cancel the page starts (its
// AbortSignal, closing the tab, navigating away) reaches the app as the
// extension's `cancel`, and the window says "site cancelled" before it goes
// (docs/ux.md §4.8), so the person knows why.
test.describe("cancel", () => {
  test("Cancel in the window rejects with UserCancelled", async () => {
    const session = await launch(env, { confirm: "cancel" });
    try {
      const page = await session.open();
      await ready(page);
      const reply = await sign(page, { hash: "SHA-512", message: newMessage() });
      expect(reply).toMatchObject({ ok: false, code: "UserCancelled" });
    } finally {
      await session.close();
    }
  });

  test("the page aborts: Aborted, and the window says the site cancelled", async () => {
    test.skip(!env.window, NEEDS_WINDOW);
    forget(env, "continue-new-site");
    forget(env, "site-cancelled");
    const session = await launch(env, { confirm: "wait" });
    try {
      const page = await session.open();
      await ready(page);
      await start(page, { hash: "SHA-256", message: newMessage() });
      await waitForState(env, "continue-new-site");
      await abort(page);
      expect(await result(page)).toMatchObject({ ok: false, code: "Aborted", prepared: [] });
      await waitForState(env, "site-cancelled");
    } finally {
      await session.close();
    }
  });

  const leave: ReadonlyArray<readonly [string, (page: Page) => Promise<unknown>]> = [
    ["closing the tab", (page) => page.close()],
    // Another site: the extension sees the tab's new origin (tabs.onUpdated).
    [
      "navigating to another site",
      async (page) => {
        const elsewhere = await startServer();
        try {
          await page.goto(`${elsewhere.origin}/`);
        } finally {
          await elsewhere.close();
        }
      },
    ],
  ];
  for (const [how, away] of leave) {
    test(`${how} ends its request with "site cancelled"`, async () => {
      // The request waits for its digest when the page goes; the next one
      // (another tab) only gets the window if the first was ended.
      forget(env, "preparing");
      forget(env, "site-cancelled");
      const session = await launch(env, { confirm: "sign" });
      try {
        const other = await session.context.newPage();
        const page = await session.open();
        await ready(page);
        await start(page, { hash: "SHA-256", message: newMessage(), holdPrepare: true });
        await prepareCalled(page);
        if (env.window) await waitForState(env, "preparing");
        await away(page);
        if (env.window) await waitForState(env, "site-cancelled");
        await other.goto(`${session.server.origin}/`);
        await ready(other);
        const key = keyOf("EC");
        const message = newMessage();
        const reply = await sign(other, { hash: "SHA-256", certificate: key.fingerprint, message });
        expectSigned(reply, key, "SHA-256", "ECDSA", message);
      } finally {
        await session.close();
      }
    });
  }
});
