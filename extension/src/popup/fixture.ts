/**
 * Screenshot fixtures: answers the popup's questions from the page's query
 * string instead of the background, so `scripts/screenshots.ts` can show
 * every state without an installed app.
 *
 * Only `main.ts` imports this, behind `import.meta.env.MODE === "fixtures"`;
 * production builds fold that to `false` and drop the module (the size
 * budget and a grep in the screenshot script both check it).
 */

import { browser } from "wxt/browser";
import type { ProbeResult } from "../shared/runtime-messages";

/** Probe answer per `?state=`; `checking` never answers. */
const PROBES: Readonly<Record<string, ProbeResult>> = {
  ready: { ok: true, appVersion: "1.4.0" },
  missing: { ok: false, code: "AppMissing" },
  outdated: { ok: false, code: "AppOutdated", details: { installed: "1.1.0", required: "1.3.0" } },
  error: { ok: false, code: "Timeout" },
};

/**
 * Replaces this page's `runtime.getPlatformInfo` and `runtime.sendMessage`.
 * `?state=` picks the answer (unsupported reports Android), `?os=` the
 * platform (win, mac, linux).
 */
export function installFixture(search: string): void {
  const query = new URLSearchParams(search);
  const state = query.get("state") ?? "ready";
  const os = state === "unsupported" ? "android" : (query.get("os") ?? "win");
  const runtime = browser.runtime as unknown as Record<string, unknown>;
  runtime.getPlatformInfo = async () => ({ os, arch: "x86-64", nacl_arch: "x86-64" });
  runtime.sendMessage = (): Promise<ProbeResult> => {
    const answer = PROBES[state];
    return answer === undefined ? new Promise(() => {}) : Promise.resolve(answer);
  };
}
