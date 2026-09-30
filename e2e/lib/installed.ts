/**
 * Registration as a person's install does it, for a named browser
 * (WEBSIGN_E2E_BROWSER_NAME): `websign register --browser <name>` with no
 * test-only folder, into the folders or registry keys that browser reads.
 *
 * - Linux: the run's private home and config home (`appEnv` sets HOME and
 *   XDG_CONFIG_HOME), so nothing of the real user is touched.
 * - Windows: `HKCU` keys (removed afterwards) pointing to manifests in a
 *   throwaway folder.
 * - macOS: the real `~/Library/Application Support` folders, because the app
 *   finds the home through the user database. Only in CI (throwaway
 *   runners); a folder already there is moved aside and put back.
 *
 * Chromium browsers read the host manifest next to their profile, and Brave
 * and Opera read their default folder whatever profile they run, so the
 * browser's default folder must be its profile. Chrome refuses DevTools on
 * its default folder, though; the default folder is therefore a symbolic
 * link to a throwaway profile: the same directory under two names.
 */

import { execFile } from "node:child_process";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  renameSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { platform, tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { promisify } from "node:util";

import { appEnv, runHome } from "./app.ts";
import { type BrowserName, browserRoot, engineOf } from "./browsers.ts";
import type { E2eEnvironment } from "./environment.ts";

const run = promisify(execFile);

/** A registration in place, with the profile the browser must run. */
export interface Installed {
  readonly profile: string;
  /** Unregisters and removes what `install` created. */
  remove(): Promise<void>;
}

/** Registers the app for `name` and prepares its profile. */
export async function install(env: E2eEnvironment, name: BrowserName): Promise<Installed> {
  if (platform() === "darwin" && process.env.CI === undefined) {
    throw new Error(
      "WEBSIGN_E2E_BROWSER_NAME on macOS writes your real browser folders; it runs in CI only",
    );
  }
  const profile = mkdtempSync(join(tmpdir(), "websign-e2e-profile-"));
  const root = browserRoot(name, {
    home: join(runHome(), "home"),
    config: join(runHome(), "config"),
  });
  const restore = root === undefined ? () => undefined : claim(root, name, profile);
  const args = ["register", "--browser", name, "--json"];
  const windows = platform() === "win32" ? ["--manifest-dir", profile] : [];
  const remove = async () => {
    await run(env.app, [...args, "--uninstall", ...windows], { env: appEnv(env, "wait") }).catch(
      () => undefined,
    );
    restore();
    rmSync(profile, { recursive: true, force: true });
  };
  try {
    await run(env.app, [...args, ...windows], { env: appEnv(env, "wait") });
  } catch (error) {
    await remove();
    throw error;
  }
  return { profile, remove };
}

/**
 * Makes `root` (the folder `register` requires to exist) exist: a link to
 * `profile` for Chromium browsers, a folder for Firefox. Returns how to undo it.
 */
function claim(root: string, name: BrowserName, profile: string): () => void {
  const aside = `${root}.websign-e2e-aside`;
  const existed = exists(root);
  if (existed) renameSync(root, aside);
  mkdirSync(dirname(root), { recursive: true });
  if (engineOf(name) === "chromium") symlinkSync(profile, root, "dir");
  else mkdirSync(root);
  return () => {
    rmSync(root, { recursive: true, force: true });
    if (existed) renameSync(aside, root);
  };
}

/** Whether `path` exists, a dangling link included. */
function exists(path: string): boolean {
  try {
    lstatSync(path);
    return true;
  } catch {
    return existsSync(path);
  }
}
