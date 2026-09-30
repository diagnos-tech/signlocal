/**
 * What the confirmation window shows, seen from outside: the e2e build
 * saves every state it shows as `confirm-<state>-<theme>.png` (the dark
 * picture last), once per app process. A scenario forgets a state's
 * pictures, acts, and waits for them to come back — a real condition,
 * never a fixed sleep.
 */

import { existsSync, rmSync } from "node:fs";
import { join } from "node:path";

import { expect } from "@playwright/test";

import type { E2eEnvironment } from "./environment.ts";

/** A state's name as the e2e build writes it (`app/src/e2e/shadow.rs`). */
export type WindowState = "continue-new-site" | "preparing" | "site-cancelled";

function picture(env: E2eEnvironment, state: WindowState, theme: "light" | "dark"): string {
  return join(env.screenshots, `confirm-${state}-${theme}.png`);
}

/** Removes the pictures of `state`, so only a new showing brings them back. */
export function forget(env: E2eEnvironment, state: WindowState): void {
  for (const theme of ["light", "dark"] as const) {
    rmSync(picture(env, state, theme), { force: true });
  }
}

/** Waits until the window has shown `state` (both pictures saved). */
export async function waitForState(
  env: E2eEnvironment,
  state: WindowState,
  timeout = 30_000,
): Promise<void> {
  await expect
    .poll(() => existsSync(picture(env, state, "dark")), {
      message: `the confirmation window never showed "${state}"`,
      timeout,
    })
    .toBe(true);
}
