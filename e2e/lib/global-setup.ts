/**
 * Once per run, before any test: a private folder for the app's data and
 * log on Linux (WEBSIGN_E2E_HOME), no renderer marker left from an earlier
 * run, the SoftHSM2 token when nothing else provides keys, and the token's
 * module named in the app's settings.
 * Variables set here reach every test (Playwright starts its workers after).
 * A folder made here is removed after the run; set WEBSIGN_E2E_HOME to keep
 * it (the app's log is in `state/websign`).
 */

import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { platform, tmpdir } from "node:os";
import { join } from "node:path";

import { addUserModule, appDataDir } from "./app.ts";
import { environment, REPO } from "./environment.ts";

const FIXTURE = join(REPO, "crates/websign-keystores/tests/support/softhsm-fixture.sh");

export default function globalSetup(): () => void {
  const given = process.env.WEBSIGN_E2E_HOME;
  const home = given ?? mkdtempSync(join(tmpdir(), "websign-e2e-home-"));
  process.env.WEBSIGN_E2E_HOME = home;
  const teardown = () => {
    if (given === undefined) rmSync(home, { recursive: true, force: true });
  };
  // The app's data folder outlives runs on macOS and Windows: a "glow
  // needed" marker from an earlier run would silently change the renderer
  // under test (app/src/ui/renderer.rs).
  rmSync(join(appDataDir(), "renderer-glow"), { force: true });
  const env = process.env;
  if (env.WEBSIGN_E2E_SOFTHSM === undefined && env.WEBSIGN_E2E_CERTS === undefined) {
    if (platform() === "win32") return teardown;
    const dir = join(home, "softhsm");
    execFileSync("bash", [FIXTURE, dir], { stdio: "inherit" });
    env.WEBSIGN_E2E_SOFTHSM = dir;
  }
  const keys = environment().keys;
  if (keys.kind === "softhsm") addUserModule(keys.token.module);
  return teardown;
}
