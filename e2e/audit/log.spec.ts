import { expect, test } from "@playwright/test";

import { appLog } from "../lib/app.ts";
import { keys } from "../lib/suite.ts";

// Scenario 8 (docs/architecture/testing.md §5), after every other scenario
// (project dependency): the app's log names nobody.
test("the log contains no certificate holder name", () => {
  const log = appLog();
  expect(log.length).toBeGreaterThan(0);
  for (const key of keys) {
    expect(log.includes(key.holder), `the log names ${key.name}'s holder`).toBe(false);
  }
});
