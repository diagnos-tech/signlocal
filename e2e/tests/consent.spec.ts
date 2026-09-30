import { createHash } from "node:crypto";

import { expect, test } from "@playwright/test";

import { launch } from "../lib/browser.ts";
import { certificates, newMessage, ready, sign, status } from "../lib/fixture.ts";
import { startServer } from "../lib/server.ts";
import { env, NEEDS_WINDOW } from "../lib/suite.ts";

// Scenarios 2 and 3 (docs/architecture/testing.md §5): who gets a
// certificate, and when (D11).
test.describe("consent", () => {
  test("certificates() on a new site asks in choose mode", async () => {
    const session = await launch(env, { confirm: "choose" });
    try {
      const page = await session.open();
      await ready(page);
      const reply = await certificates(page);
      if (!reply.ok) throw new Error(`certificates() failed with ${reply.code}`);
      const der = Buffer.from(reply.der, "base64");
      expect(reply.fingerprint).toBe(createHash("sha256").update(der).digest("hex"));
    } finally {
      await session.close();
    }
  });

  test("D11: a new site cancelled before Continue gets no certificate", async () => {
    const session = await launch(env, { confirm: "cancel" });
    try {
      const page = await session.open();
      await ready(page);
      const signed = await sign(page, { hash: "SHA-256", message: newMessage() });
      expect(signed).toEqual({ ok: false, code: "UserCancelled", prepared: [] });
      expect(await certificates(page)).toEqual({ ok: false, code: "UserCancelled" });
    } finally {
      await session.close();
    }
  });

  test("a remembered site gets certificates() without a window", async () => {
    test.skip(!env.window, NEEDS_WINDOW);
    const server = await startServer();
    try {
      const first = await launch(env, { confirm: "remember", server });
      try {
        const page = await first.open();
        await ready(page);
        expect(await certificates(page)).toMatchObject({ ok: true });
        expect(await status(page)).toMatchObject({ remembered: true, ready: true });
      } finally {
        await first.close();
      }
      // Nobody answers a window now: only an answer without one can arrive.
      const second = await launch(env, { confirm: "wait", server });
      try {
        const page = await second.open();
        await ready(page);
        const reply = await Promise.race([
          certificates(page),
          new Promise<"window">((done) => setTimeout(() => done("window"), 20_000)),
        ]);
        expect(reply).toMatchObject({ ok: true });
      } finally {
        await second.close();
      }
    } finally {
      await server.close();
    }
  });
});
