/**
 * The app under test: the environment its processes get (keys, isolated
 * data and log folders on Linux, the e2e hooks) and the commands the suite
 * runs directly (`register`, `diagnostics`).
 */

import { execFile } from "node:child_process";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir, platform } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";

import type { E2eEnvironment } from "./environment.ts";

const run = promisify(execFile);

/** What the confirmation window does by itself (`app/src/e2e.rs`). */
export type Confirm = "sign" | "choose" | "remember" | "cancel" | "wait";

/** Per-run folders, made once by `global-setup.ts` (WEBSIGN_E2E_HOME). */
function runHome(): string {
  const home = process.env.WEBSIGN_E2E_HOME;
  if (home === undefined) throw new Error("WEBSIGN_E2E_HOME is not set: run through playwright");
  return home;
}

/** The app's settings and data folder (`dirs::config_dir()/websign`). */
export function appDataDir(): string {
  switch (platform()) {
    case "win32":
      return join(process.env.APPDATA ?? join(homedir(), "AppData", "Roaming"), "websign");
    case "darwin":
      return join(homedir(), "Library", "Application Support", "websign");
    default:
      return join(runHome(), "config", "websign");
  }
}

/** The app's log folder (`app/src/logging/location.rs`). */
export function appLogDir(): string {
  switch (platform()) {
    case "win32":
      return join(
        process.env.LOCALAPPDATA ?? join(homedir(), "AppData", "Local"),
        "websign",
        "logs",
      );
    case "darwin":
      return join(homedir(), "Library", "Logs", "websign");
    default:
      return join(runHome(), "state", "websign");
  }
}

/** Environment for every app process (started by the browser or by us). */
export function appEnv(
  env: E2eEnvironment,
  confirm: Confirm,
  screenshots = true,
): NodeJS.ProcessEnv {
  const vars: NodeJS.ProcessEnv = {
    ...process.env,
    WEBSIGN_LOCALE: "en",
    WEBSIGN_LOG: process.env.WEBSIGN_LOG ?? "info",
  };
  if (platform() === "linux") {
    vars.XDG_CONFIG_HOME = join(runHome(), "config");
    vars.XDG_STATE_HOME = join(runHome(), "state");
  }
  if (env.keys.kind === "softhsm") {
    vars.SOFTHSM2_CONF = env.keys.token.conf;
    vars.WEBSIGN_E2E_PIN = env.keys.token.pin;
  }
  if (confirm !== "wait") vars.WEBSIGN_E2E_CONFIRM = confirm;
  if (!env.window) vars.WEBSIGN_E2E_HEADLESS = "1";
  // Each window state is saved once per app process, so this costs little.
  if (screenshots && env.window) vars.WEBSIGN_E2E_SCREENSHOTS = env.screenshots;
  return vars;
}

/** Names the SoftHSM2 module as a driver the person added, keeping other settings. */
export function addUserModule(module: string): void {
  const dir = appDataDir();
  mkdirSync(dir, { recursive: true });
  const file = join(dir, "settings.json");
  let settings: { version?: number; userModules?: string[] } = { version: 1 };
  try {
    settings = JSON.parse(readFileSync(file, "utf8"));
  } catch {
    // No settings yet.
  }
  const modules = new Set(settings.userModules ?? []);
  modules.add(module);
  writeFileSync(file, JSON.stringify({ ...settings, userModules: [...modules] }));
}

/**
 * Registers the app as the native messaging host for a Chromium profile:
 * in the profile itself on Linux and macOS, in HKCU (manifest in the
 * profile folder) on Windows.
 */
export async function register(
  env: E2eEnvironment,
  profile: string,
  uninstall = false,
): Promise<void> {
  const args = ["register", "--browser", "chromium", "--json"];
  if (uninstall) args.push("--uninstall");
  args.push(
    ...(platform() === "win32" ? ["--manifest-dir", profile] : ["--user-data-dir", profile]),
  );
  await run(env.app, args, { env: appEnv(env, "wait") });
}

/** Opens diagnostics on `tab`; the e2e hook saves it and closes the window. */
export async function diagnostics(env: E2eEnvironment, tab: string): Promise<void> {
  await run(env.app, ["diagnostics", "--tab", tab], {
    env: appEnv(env, "wait", true),
    timeout: 60_000,
  });
}

/** Every line the app logged in this run. */
export function appLog(): string {
  try {
    const dir = appLogDir();
    return readdirSync(dir)
      .map((file) => readFileSync(join(dir, file), "utf8"))
      .join("\n");
  } catch {
    return "";
  }
}
