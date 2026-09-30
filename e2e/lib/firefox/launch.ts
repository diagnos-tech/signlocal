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
import { readFileSync, writeFileSync } from "node:fs";
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
  readonly version: string;
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
    const bidi = await Bidi.connect(await listening(firefox, installed.profile));
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
      version: bidi.version,
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

/**
 * The BiDi address Firefox writes to `WebDriverBiDiServer.json` in the
 * profile once it listens. The file, not stderr: a Windows Firefox is a GUI
 * program whose console output is not reliable. Stderr is still drained (a
 * full pipe would stall Firefox) and kept for the error message.
 */
async function listening(firefox: ChildProcess, profile: string): Promise<string> {
  let seen = "";
  firefox.stderr?.on("data", (chunk: Buffer) => {
    seen = (seen + chunk.toString()).slice(-8000);
  });
  const file = join(profile, "WebDriverBiDiServer.json");
  const deadline = Date.now() + START_TIMEOUT;
  while (Date.now() < deadline) {
    if (firefox.exitCode !== null) break;
    try {
      const server = JSON.parse(readFileSync(file, "utf8")) as { ws_host: string; ws_port: number };
      return `ws://${server.ws_host}:${server.ws_port}`;
    } catch {
      // Not written yet (or half written).
    }
    await new Promise((done) => setTimeout(done, 200));
  }
  throw new Error(`Firefox did not start listening (exit ${firefox.exitCode}):\n${seen}`);
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
