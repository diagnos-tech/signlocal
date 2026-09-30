/**
 * The browser under test behind one small interface, for the cross-browser
 * suite (`browsers/`): Chromium-family browsers through Playwright
 * (`browser.ts`), Firefox over WebDriver BiDi (`firefox/launch.ts`).
 */

import type { Confirm } from "./app.ts";
import { launch } from "./browser.ts";
import { engineOf } from "./browsers.ts";
import type { E2eEnvironment } from "./environment.ts";
import { launchFirefox } from "./firefox/launch.ts";
import type { Tab } from "./fixture.ts";

/** A running browser with the extension, the app registered for it. */
export interface BrowserSession {
  /** Shows the fixture page (or `path` on the page server). */
  open(path?: string): Promise<Tab>;
  close(): Promise<void>;
}

/** Launches the run's browser (`env.browserName`, else Playwright's Chromium). */
export async function openBrowser(env: E2eEnvironment, confirm: Confirm): Promise<BrowserSession> {
  if (env.browserName !== undefined && engineOf(env.browserName) === "firefox") {
    const firefox = await launchFirefox(env, confirm);
    return { open: (path) => firefox.open(path), close: () => firefox.close() };
  }
  const session = await launch(env, { confirm });
  return { open: (path) => session.open(path), close: () => session.close() };
}
