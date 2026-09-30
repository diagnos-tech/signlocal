/**
 * The installed browsers a run can target (WEBSIGN_E2E_BROWSER_NAME) and
 * where each keeps its default profile. A named browser is registered the
 * way `websign install` registers it for a person (`websign register
 * --browser <name>`, no test-only folder), so a passing run proves the real
 * manifest location or registry key, not only the protocol.
 */

import { homedir, platform } from "node:os";
import { join } from "node:path";

/** A `websign register --browser` value the suite can drive. */
export type BrowserName =
  | "chrome"
  | "chromium"
  | "edge"
  | "brave"
  | "vivaldi"
  | "opera"
  | "firefox";

export const BROWSER_NAMES: readonly BrowserName[] = [
  "chrome",
  "chromium",
  "edge",
  "brave",
  "vivaldi",
  "opera",
  "firefox",
];

/** Engines load the extension and drive pages differently. */
export type Engine = "chromium" | "firefox";

export function engineOf(name: BrowserName): Engine {
  return name === "firefox" ? "firefox" : "chromium";
}

/** Linux: folder under the config home (`$XDG_CONFIG_HOME`); Firefox's is in the home. */
const LINUX: Readonly<Record<Exclude<BrowserName, "firefox">, string>> = {
  chrome: "google-chrome",
  chromium: "chromium",
  edge: "microsoft-edge",
  brave: "BraveSoftware/Brave-Browser",
  vivaldi: "vivaldi",
  opera: "opera",
};

/** macOS: folder under ~/Library/Application Support. */
const MACOS: Readonly<Record<BrowserName, string>> = {
  chrome: "Google/Chrome",
  chromium: "Chromium",
  edge: "Microsoft Edge",
  brave: "BraveSoftware/Brave-Browser",
  vivaldi: "Vivaldi",
  opera: "com.operasoftware.Opera",
  firefox: "Mozilla",
};

/**
 * The folder whose existence tells `websign register` the browser is
 * installed, and the browser's default profile folder (Chromium browsers on
 * Linux and macOS read the host manifest next to their profile, and Brave and
 * Opera read a fixed folder whatever profile they run: Brave its default
 * folder on Linux, Opera Google Chrome's, and on macOS Brave Google Chrome's
 * too, which registration writes whenever Brave's own folder exists). `undefined`
 * on Windows, where registration is a registry key and any profile works.
 *
 * Linux: under the run's private home and config home (`linux`), nothing
 * real is touched. macOS: the real folder under the user's home, because the
 * app finds the home through the user database; `installed.ts` refuses it
 * outside CI.
 */
export function browserRoot(
  name: BrowserName,
  linux: { readonly home: string; readonly config: string },
): string | undefined {
  switch (platform()) {
    case "win32":
      return undefined;
    case "darwin":
      return join(homedir(), "Library", "Application Support", MACOS[name]);
    default:
      return name === "firefox" ? join(linux.home, ".mozilla") : join(linux.config, LINUX[name]);
  }
}
