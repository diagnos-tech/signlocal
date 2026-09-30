/**
 * Firefox with the extension installed as a temporary add-on (BiDi
 * `webExtension.install`, which accepts the unsigned development build) in
 * a throwaway profile, and the app registered as a person's install does
 * (`installed.ts`: `~/.mozilla/native-messaging-hosts` on Linux, the
 * `HKCU\Software\Mozilla` key on Windows, `Mozilla/NativeMessagingHosts` on
 * macOS). Firefox hands its environment to the app it starts, as Chromium
 * does, so each launch fixes what the confirmation window does.
 */

import { type ChildProcess, spawn } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";

import { appEnv, type Confirm } from "../app.ts";
import type { E2eEnvironment } from "../environment.ts";
import { install } from "../installed.ts";
import { type PageServer, startServer } from "../server.ts";
import { Bidi } from "./bidi.ts";
import { FirefoxTab } from "./tab.ts";

/** How long Firefox may take to start listening. */
const START_TIMEOUT = 60_000;

/**
 * Preferences of the throwaway profile: no first-run pages or default-browser
 * prompt over the app's window, and native messaging started directly (the
 * portal is for Snap and Flatpak builds, which CI does not run).
 */
const PREFS: Readonly<Record<string, boolean | string | number>> = {
  "browser.shell.checkDefaultBrowser": false,
  "browser.startup.homepage_override.mstone": "ignore",
  "browser.aboutwelcome.enabled": false,
  "datareporting.policy.dataSubmissionEnabled": false,
  "toolkit.telemetry.reportingpolicy.firstRun": false,
  "widget.use-xdg-desktop-portal.native-messaging": 0,
};

/** A running Firefox with a page server. */
export interface FirefoxSession {
  readonly server: PageServer;
  /** Shows the fixture page (or `path` on the server) in the session's tab. */
  open(path?: string): Promise<FirefoxTab>;
  close(): Promise<void>;
}

/** Starts Firefox for `env.browser` with the window acting as `confirm` says. */
export async function launchFirefox(
  env: E2eEnvironment,
  confirm: Confirm,
): Promise<FirefoxSession> {
  if (env.browser === undefined) throw new Error("WEBSIGN_E2E_BROWSER must name firefox");
  const installed = await install(env, "firefox");
  const prefs = Object.entries(PREFS).map(
    ([key, value]) => `user_pref(${JSON.stringify(key)}, ${JSON.stringify(value)});`,
  );
  writeFileSync(join(installed.profile, "user.js"), `${prefs.join("\n")}\n`);
  const firefox = spawn(
    env.browser,
    ["--remote-debugging-port=0", "--profile", installed.profile, "--no-remote", "--new-instance"],
    { env: appEnv(env, confirm), stdio: ["ignore", "ignore", "pipe"] },
  );
  const stop = async () => {
    await exit(firefox);
    await installed.remove();
  };
  try {
    const bidi = await Bidi.connect(await listening(firefox));
    await bidi.send("webExtension.install", {
      extensionData: { type: "path", path: env.extension },
    });
    const tree = await bidi.send<{ contexts: ReadonlyArray<{ context: string }> }>(
      "browsingContext.getTree",
      { maxDepth: 0 },
    );
    const first = tree.contexts[0];
    if (first === undefined) throw new Error("Firefox opened no tab");
    const tab = new FirefoxTab(bidi, first.context);
    const server = await startServer();
    return {
      server,
      async open(path = "/") {
        await tab.goto(server.origin + path);
        return tab;
      },
      async close() {
        await bidi.close();
        await server.close();
        await stop();
      },
    };
  } catch (error) {
    await stop();
    throw error;
  }
}

/** The BiDi address Firefox prints on stderr once it listens. */
function listening(firefox: ChildProcess): Promise<string> {
  return new Promise((resolve, reject) => {
    let seen = "";
    const timer = setTimeout(
      () => reject(new Error(`Firefox did not start listening:\n${seen.slice(-2000)}`)),
      START_TIMEOUT,
    );
    // Kept draining after the address is found: a full pipe would stall Firefox.
    firefox.stderr?.on("data", (chunk: Buffer) => {
      seen = (seen + chunk.toString()).slice(-8000);
      const found = /WebDriver BiDi listening on (ws:\/\/\S+)/.exec(seen);
      if (found?.[1] !== undefined) {
        clearTimeout(timer);
        resolve(found[1]);
      }
    });
    firefox.once("exit", (code) => {
      clearTimeout(timer);
      reject(new Error(`Firefox exited (${code}) before listening:\n${seen.slice(-2000)}`));
    });
  });
}

/** Ends Firefox, forcefully when it does not quit in time. */
async function exit(firefox: ChildProcess): Promise<void> {
  if (firefox.exitCode !== null || firefox.signalCode !== null) return;
  const exited = new Promise<void>((done) => firefox.once("exit", () => done()));
  firefox.kill();
  const timer = setTimeout(() => firefox.kill("SIGKILL"), 10_000);
  await exited;
  clearTimeout(timer);
}
