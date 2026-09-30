import { expect, test } from "@playwright/test";

import { launch } from "../lib/browser.ts";
import { newMessage, ready, sign, start } from "../lib/fixture.ts";
import { env, expectSigned, keyOf, NEEDS_WINDOW } from "../lib/suite.ts";

// Scenario 5 (docs/architecture/testing.md §5): the person cancels in the
// window, or the page gives up (abort, tab closed). Both reach the app as
// the extension's `cancel`, which ends the request without a notice.
// TODO(gustavo): testing.md §5 expects "site cancelled" on screen for a
// closed tab; the host shows it only when the whole connection drops.
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

  test("the page aborts: Aborted, and the window moves on", async () => {
    // A withdrawn request closes without a notice (Finish::Aborted).
    test.skip(!env.window, NEEDS_WINDOW);
    const session = await launch(env, { confirm: "wait" });
    try {
      const page = await session.open();
      await ready(page);
      const reply = await sign(page, { hash: "SHA-256", message: newMessage(), abortAfter: 4_000 });
      expect(reply).toMatchObject({ ok: false, code: "Aborted", prepared: [] });
    } finally {
      await session.close();
    }
  });

  test("closing the tab ends its request in the app", async () => {
    // The request waits for its digest when the tab closes; the next one
    // (another tab) only gets the window if the first was ended.
    const session = await launch(env, { confirm: "sign" });
    try {
      const other = await session.context.newPage();
      const page = await session.open();
      await ready(page);
      await start(page, { hash: "SHA-256", message: newMessage(), holdPrepare: true });
      await page.waitForTimeout(6_000);
      await page.close();
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
});
