import { expect, test } from "@playwright/test";

import { appLog } from "../lib/app.ts";
import { keys } from "../lib/suite.ts";

// Scenario 8 (docs/architecture/testing.md §5), after every other scenario
// (project dependency): the app's log names nobody and no certificate.
test("the log contains no holder name and no certificate fingerprint", () => {
  const log = appLog().toLowerCase();
  expect(log.length, "the app wrote no log in this run").toBeGreaterThan(0);
  for (const key of keys) {
    expect(log.includes(key.holder.toLowerCase()), `the log names ${key.name}'s holder`).toBe(
      false,
    );
    expect(log.includes(key.fingerprint), `the log has ${key.name}'s fingerprint`).toBe(false);
  }
});
