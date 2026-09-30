/**
 * What an e2e run needs, from the environment set up by CI (or a developer):
 * the e2e app build, the unpacked extension, the software keys and where
 * screenshots go.
 *
 * How each OS receives its software keys (the app finds them as it would a
 * person's; the suite only needs their certificates to pick and verify):
 *
 * - Linux and containers: a SoftHSM2 token made by
 *   `crates/websign-keystores/tests/support/softhsm-fixture.sh`. Set
 *   WEBSIGN_E2E_SOFTHSM to its folder, or leave it unset and the suite makes
 *   one (`global-setup.ts`). The app loads the module as a driver the person
 *   added (`userModules` in its settings.json) and reads SOFTHSM2_CONF.
 * - Windows: CNG and CAPI software keys in Cert:\CurrentUser\My made by
 *   `docs/prototypes/kit/windows/make-test-certs.ps1`; CI exports their
 *   certificates to WEBSIGN_E2E_CERTS.
 * - macOS: identities imported into a temporary keychain on the search list
 *   (`docs/prototypes/kit/macos/lib/common.sh`), the e2e binary allowed in
 *   its partition list; CI copies their certificates to WEBSIGN_E2E_CERTS.
 *
 * WEBSIGN_E2E_BROWSER_NAME names an installed browser to prove
 * (`lib/browsers.ts`); the suite then registers the host as a person's
 * install does.
 *
 * WEBSIGN_E2E_WINDOW=headless is for machines without a display: the app
 * then decides without its window (`WEBSIGN_E2E_HEADLESS=1`) and the
 * scenarios about the window are skipped.
 */

import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { BROWSER_NAMES, type BrowserName } from "./browsers.ts";

/** The repository root. */
export const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

/** A SoftHSM2 token made by softhsm-fixture.sh. */
export interface SoftHsm {
  readonly dir: string;
  readonly conf: string;
  readonly module: string;
  readonly pin: string;
}

/** Where the app's software keys are. */
export type KeySource =
  | { readonly kind: "softhsm"; readonly token: SoftHsm }
  | { readonly kind: "os-store"; readonly certificates: string };

/** The e2e environment. */
export interface E2eEnvironment {
  /** `websign` built with `--features e2e` (WEBSIGN_E2E_APP). */
  readonly app: string;
  /**
   * Unpacked extension (WEBSIGN_E2E_EXTENSION), by default the build for
   * the browser's engine: extension/.output/chrome-mv3 or firefox-mv3.
   */
  readonly extension: string;
  /** Screenshot folder (WEBSIGN_E2E_SCREENSHOTS). */
  readonly screenshots: string;
  /** Browser executable to use instead of Playwright's (WEBSIGN_E2E_BROWSER). */
  readonly browser?: string;
  /**
   * The installed browser this run proves (WEBSIGN_E2E_BROWSER_NAME, with
   * WEBSIGN_E2E_BROWSER its executable): registered as `websign install`
   * does, and only the cross-browser suite (`browsers/`) runs. Unset: the
   * full suite in Playwright's Chromium (or WEBSIGN_E2E_BROWSER), with the
   * host registered in the throwaway profile.
   */
  readonly browserName?: BrowserName;
  readonly keys: KeySource;
  /** The confirmation window opens (false: WEBSIGN_E2E_WINDOW=headless). */
  readonly window: boolean;
}

/** Reads the environment, failing with a clear message when something is missing. */
export function environment(): E2eEnvironment {
  const env = process.env;
  const app = required("WEBSIGN_E2E_APP", "the `websign` binary built with --features e2e");
  const browserName = named(env.WEBSIGN_E2E_BROWSER_NAME);
  const build = browserName === "firefox" ? "firefox-mv3" : "chrome-mv3";
  const extension = env.WEBSIGN_E2E_EXTENSION ?? join(REPO, "extension/.output", build);
  mustExist(extension, `WEBSIGN_E2E_EXTENSION (build it with \`bunx wxt build\`: ${build})`);
  const screenshots = env.WEBSIGN_E2E_SCREENSHOTS ?? join(tmpdir(), "websign-e2e-screenshots");
  mkdirSync(screenshots, { recursive: true });
  const browser = env.WEBSIGN_E2E_BROWSER;
  if (browser !== undefined) mustExist(browser, "WEBSIGN_E2E_BROWSER");
  if (browserName !== undefined && browser === undefined) {
    throw new Error(
      "WEBSIGN_E2E_BROWSER_NAME needs WEBSIGN_E2E_BROWSER (the browser's executable)",
    );
  }
  return {
    app,
    extension,
    screenshots,
    ...(browser === undefined ? {} : { browser }),
    ...(browserName === undefined ? {} : { browserName }),
    keys: keySource(),
    window: env.WEBSIGN_E2E_WINDOW !== "headless",
  };
}

function named(value: string | undefined): BrowserName | undefined {
  if (value === undefined || value === "") return undefined;
  const name = BROWSER_NAMES.find((known) => known === value);
  if (name === undefined) {
    throw new Error(
      `WEBSIGN_E2E_BROWSER_NAME=${value}: expected one of ${BROWSER_NAMES.join(", ")}`,
    );
  }
  return name;
}

function keySource(): KeySource {
  const softhsm = process.env.WEBSIGN_E2E_SOFTHSM;
  if (softhsm !== undefined) return { kind: "softhsm", token: readToken(softhsm) };
  const certificates = process.env.WEBSIGN_E2E_CERTS;
  if (certificates !== undefined) {
    mustExist(certificates, "WEBSIGN_E2E_CERTS");
    return { kind: "os-store", certificates };
  }
  throw new Error(
    "No software keys: set WEBSIGN_E2E_SOFTHSM (a softhsm-fixture.sh folder) or WEBSIGN_E2E_CERTS " +
      "(certificates of the OS store's test keys); see e2e/lib/environment.ts",
  );
}

/** Reads `fixture.conf` (KEY=VALUE lines) of a softhsm-fixture.sh folder. */
export function readToken(dir: string): SoftHsm {
  const conf = join(dir, "fixture.conf");
  mustExist(conf, "WEBSIGN_E2E_SOFTHSM/fixture.conf");
  const values = new Map<string, string>();
  for (const line of readFileSync(conf, "utf8").split(/\r?\n/)) {
    const at = line.indexOf("=");
    if (at > 0) values.set(line.slice(0, at), line.slice(at + 1));
  }
  const get = (key: string): string => {
    const value = values.get(key);
    if (value === undefined) throw new Error(`${conf} has no ${key}`);
    return value;
  };
  return {
    dir,
    conf: get("SOFTHSM2_CONF"),
    module: get("SOFTHSM_MODULE"),
    pin: get("SOFTHSM_USER_PIN"),
  };
}

function required(name: string, what: string): string {
  const value = process.env[name];
  if (value === undefined || value === "") throw new Error(`${name} is not set: ${what}`);
  mustExist(value, name);
  return value;
}

function mustExist(path: string, what: string): void {
  if (!existsSync(path)) throw new Error(`${what}: ${path} does not exist`);
}
