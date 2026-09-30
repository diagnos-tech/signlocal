/**
 * Chromium with the unpacked extension in a fresh profile, the e2e app
 * registered as that profile's native messaging host (proven in
 * `docs/prototypes/kit/nm-e2e`). The browser hands its environment to the
 * app it starts, so each launch fixes what the confirmation window does.
 */

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { type BrowserContext, chromium, type Page } from "@playwright/test";

import { appEnv, type Confirm, register } from "./app.ts";
import type { E2eEnvironment } from "./environment.ts";
import { installOldApp } from "./old-app.ts";
import { type PageServer, startServer } from "./server.ts";

/** What a launch sets up. */
export interface LaunchOptions {
  readonly confirm: Confirm;
  /** Load the extension (false: ExtensionMissing). */
  readonly extension?: boolean;
  /**
   * The native messaging host: the app (default), none (AppMissing), or
   * the fake old app (AppOutdated, `old-app.ts`).
   */
  readonly host?: "app" | "none" | "old-app";
  /** Serve the pages from this (still open) server: the same site again. */
  readonly server?: PageServer;
}

/** A running browser with a page server. */
export interface Session {
  readonly context: BrowserContext;
  readonly server: PageServer;
  /**
   * Shows the fixture page (or `path` on the server) in the session's tab.
   * One tab for the whole session: a new browser window would cover the
   * app's window on a display without a window manager (Xvfb).
   */
  open(path?: string): Promise<Page>;
  close(): Promise<void>;
}

/** Launches Chromium as `options` say. */
export async function launch(env: E2eEnvironment, options: LaunchOptions): Promise<Session> {
  const profile = mkdtempSync(join(tmpdir(), "websign-e2e-profile-"));
  const withExtension = options.extension ?? true;
  const host = options.host ?? "app";
  const withApp = host !== "none";
  if (withApp) await register(env, profile);
  if (host === "old-app") installOldApp(profile);
  const context = await chromium.launchPersistentContext(profile, {
    ...(env.browser === undefined ? {} : { executablePath: env.browser }),
    // Extensions need a headed Chromium (Xvfb on Linux CI).
    headless: false,
    env: appEnv(env, options.confirm) as Record<string, string>,
    ignoreDefaultArgs: ["--disable-extensions"],
    // Left of the app's window, which opens centered on the screen.
    args: [
      "--window-position=0,0",
      "--window-size=520,700",
      ...(withExtension
        ? [`--disable-extensions-except=${env.extension}`, `--load-extension=${env.extension}`]
        : []),
    ],
  });
  const server = options.server ?? (await startServer());
  return {
    context,
    server,
    async open(path = "/") {
      const page = context.pages().find((p) => !p.isClosed()) ?? (await context.newPage());
      await page.goto(server.origin + path);
      return page;
    },
    async close() {
      await context.close();
      if (options.server === undefined) await server.close();
      if (withApp) await register(env, profile, true).catch(() => undefined);
      rmSync(profile, { recursive: true, force: true });
    },
  };
}
