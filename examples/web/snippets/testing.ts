// "Test without the extension": the region is cut out by the developers page of the site.
// The test runner is Bun's here; Vitest and Jest read the same.

import { afterEach, beforeEach, expect, it } from "bun:test";
import type { SignResult } from "@signlocal/sdk";

declare function signContract(): Promise<SignResult>;
declare const signedAttributes: Uint8Array;

// #region site:testing
import { type FakeSignlocal, installFakeSignlocal } from "@signlocal/sdk/testing";

let fake: FakeSignlocal;
beforeEach(async () => (fake = await installFakeSignlocal())); // in jsdom or happy-dom
afterEach(() => fake.uninstall());

it("signs the contract", async () => {
  const result = await signContract(); // your code, calling sign()
  expect(await fake.verify(result, signedAttributes)).toBe(true);
});

it("stays calm when the person cancels", async () => {
  fake.failNext("UserCancelled");
  await expect(signContract()).rejects.toMatchObject({ code: "UserCancelled" });
});

// Other setups: installFakeSignlocal({ scenario: "app-missing" | "app-outdated" | "extension-missing" | … }),
// installFakeSignlocal({ certificates: [] }), fake.switchDuringNextSign(1), fake.requests.
// #endregion site:testing
