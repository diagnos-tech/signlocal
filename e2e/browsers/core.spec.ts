import { writeFileSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "@playwright/test";

import { certificates, newMessage, ready, sign, status } from "../lib/fixture.ts";
import { useBrowser } from "../lib/session.ts";
import { env, expectSigned, keyOf } from "../lib/suite.ts";

// The cross-browser core (docs/compatibility.md, "Browsers"): in every
// installed browser CI can run, registered as a person's install does, the
// extension finds the app, signs with an EC and an RSA key (each signature
// verified with node:crypto), the person's Cancel reaches the page, and a
// new site cancelled before Continue gets no certificate (D11).
const browser = env.browserName ?? "chromium";

test.describe(`${browser}: finds the app and signs`, () => {
  const session = useBrowser(env, "sign", "all");

  test("the page sees the extension and the app", async () => {
    // The version tested, for the compatibility record (`run-browsers.sh`).
    writeFileSync(join(env.screenshots, "browser-version.txt"), `${session().version}\n`);
    const page = await session().open();
    await ready(page);
    expect(await status(page)).toMatchObject({
      extension: { installed: true },
      app: { installed: true, outdated: false },
    });
  });

  const cases = [
    ["EC", "ECDSA", "SHA-256"],
    ["RSA", "RSASSA-PKCS1-v1_5", "SHA-256"],
    ["RSA", "RSASSA-PSS", "SHA-384"],
  ] as const;
  for (const [type, algorithm, hash] of cases) {
    test(`${algorithm} ${hash} with the ${type} key`, async () => {
      const key = keyOf(type);
      const page = await session().open();
      await ready(page);
      const message = newMessage();
      const reply = await sign(page, { hash, algorithm, certificate: key.fingerprint, message });
      expectSigned(reply, key, hash, algorithm, message);
    });
  }
});

test.describe(`${browser}: the person cancels`, () => {
  const session = useBrowser(env, "cancel", "each");

  test("Cancel in the window rejects with UserCancelled", async () => {
    const page = await session().open();
    await ready(page);
    const reply = await sign(page, { hash: "SHA-512", message: newMessage() });
    expect(reply).toMatchObject({ ok: false, code: "UserCancelled" });
  });

  test("D11: a new site cancelled before Continue gets no certificate", async () => {
    const page = await session().open();
    await ready(page);
    expect(await sign(page, { hash: "SHA-256", message: newMessage() })).toEqual({
      ok: false,
      code: "UserCancelled",
      prepared: [],
    });
    expect(await certificates(page)).toEqual({ ok: false, code: "UserCancelled" });
  });
});
