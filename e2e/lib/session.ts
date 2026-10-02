/**
 * The browser under test behind one small interface, for the cross-browser
 * suite (`browsers/`): Chromium-family browsers through Playwright
 * (`browser.ts`), Firefox over WebDriver BiDi (`firefox/launch.ts`).
 */

import { test } from "@playwright/test";

import type { Confirm } from "./app.ts";
import { launch } from "./browser.ts";
import { engineOf } from "./browsers.ts";
import type { E2eEnvironment } from "./environment.ts";
import { launchFirefox } from "./firefox/launch.ts";
import type { Tab } from "./fixture.ts";

/** A running browser with the extension, the app registered for it. */
export interface BrowserSession {
  /** The browser's product version, for the compatibility record. */
  readonly version: string;
  /** Shows the fixture page (or `path` on the page server). */
  open(path?: string): Promise<Tab>;
  close(): Promise<void>;
}

/** Launches the run's browser (`env.browserName`, else Playwright's Chromium). */
export async function openBrowser(env: E2eEnvironment, confirm: Confirm): Promise<BrowserSession> {
  if (env.browserName !== undefined && engineOf(env.browserName) === "firefox") {
    const firefox = await launchFirefox(env, confirm);
    return {
      version: firefox.version,
      open: (path) => firefox.open(path),
      close: () => firefox.close(),
    };
  }
  const session = await launch(env, { confirm });
  return {
    version: session.context.browser()?.version() ?? "unknown",
    open: (path) => session.open(path),
    close: () => session.close(),
  };
}

/**
 * The browser of a `describe` block, opened before all its tests or before
 * each, and closed after. A launch that fails is reported by the hook that
 * ran it; the closing hook then has nothing to close instead of failing
 * again with an unrelated error.
 */
export function useBrowser(
  env: E2eEnvironment,
  confirm: Confirm,
  per: "all" | "each",
): () => BrowserSession {
  let session: BrowserSession | undefined;
  const [open, close] =
    per === "all" ? [test.beforeAll, test.afterAll] : [test.beforeEach, test.afterEach];
  open(async () => {
    session = await openBrowser(env, confirm);
  });
  close(async () => {
    const current = session;
    session = undefined;
    await current?.close();
  });
  return () => {
    if (session === undefined) throw new Error("the browser did not start");
    return session;
  };
}
