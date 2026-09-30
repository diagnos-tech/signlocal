import { expect, test } from "@playwright/test";

import { launch, type Session } from "../lib/browser.ts";
import { newMessage, ready, sign } from "../lib/fixture.ts";
import { env, expectSigned, HASHES, keyOf, signers } from "../lib/suite.ts";
import type { SignatureAlgorithm } from "../lib/verify.ts";

// Scenario 1 (docs/architecture/testing.md §5): every hash with every
// software key and scheme, confirmed in the app's window, each signature
// verified with node:crypto.
test.describe("sign through the extension", () => {
  let session: Session;
  test.beforeAll(async () => {
    session = await launch(env, { confirm: "sign" });
  });
  test.afterAll(async () => {
    await session.close();
  });

  for (const key of signers) {
    const schemes: SignatureAlgorithm[] =
      key.type === "RSA" ? ["RSASSA-PKCS1-v1_5", "RSASSA-PSS"] : ["ECDSA"];
    for (const algorithm of schemes) {
      for (const hash of HASHES) {
        test(`${key.name} ${algorithm} ${hash}`, async () => {
          const page = await session.open();
          await ready(page);
          const message = newMessage();
          const reply = await sign(page, {
            hash,
            algorithm,
            certificate: key.fingerprint,
            message,
          });
          expectSigned(reply, key, hash, algorithm, message);
        });
      }
    }
  }

  // With no algorithm named, every key the store can use must stay signable
  // and get the first default its store produces (protocol.md §4.3): an EC
  // key reported as unable to sign would land in "Can't sign" and fail here.
  for (const key of signers) {
    const expected: SignatureAlgorithm = key.type === "RSA" ? "RSASSA-PKCS1-v1_5" : "ECDSA";
    test(`${key.name} signs when the page names no algorithm`, async () => {
      const page = await session.open();
      await ready(page);
      const message = newMessage();
      const reply = await sign(page, { hash: "SHA-384", certificate: key.fingerprint, message });
      expectSigned(reply, key, "SHA-384", expected, message);
    });
  }

  test("a digest of the wrong length is refused", async () => {
    const page = await session.open();
    await ready(page);
    const reply = await sign(page, {
      hash: "SHA-384",
      certificate: keyOf("EC").fingerprint,
      message: newMessage(),
      digestLength: 32,
    });
    expect(reply).toMatchObject({ ok: false, code: "InvalidRequest" });
  });

  test("the website's test page signs and verifies", async () => {
    const page = await session.open("/site/test/index.html");
    await expect(page.locator('#setup [data-state="ready"]')).toBeVisible();
    await page.locator("#sign").click();
    await expect(page.locator("#result-ok")).toBeVisible({ timeout: 60_000 });
    await expect(page.locator("#v-invalid")).toBeHidden();
  });
});
