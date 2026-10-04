/**
 * Shared setup of the popup DOM tests: a fresh `#app`, the fake browser with
 * messages that echo their substitutions, and helpers to show a probe result.
 */

import { vi } from "vitest";
import type { ProbeResult } from "../src/shared/runtime-messages";
import { fakeBrowser } from "./fakes/browser";

export const HOME = "https://diagnos-tech.github.io/signlocal/";

/** `tabs.create`, which the shared fake lacks; the popup opens links with it. */
export const createTab = vi.fn(async (_properties: { url: string }) => ({}));

/** getMessage that shows its substitutions, so the test sees what was filled in. */
function messages(key: string, substitutions?: string | string[]): string {
  const list = substitutions === undefined ? [] : [substitutions].flat();
  return list.length > 0 ? `${key}(${list.join(",")})` : key;
}

/** Resets the fakes and the document; call from `beforeEach`. */
export function setUpPopup(): void {
  vi.useFakeTimers();
  fakeBrowser.reset();
  fakeBrowser.manifestVersion = "1.4.2";
  fakeBrowser.i18n.getMessage.mockImplementation(messages);
  createTab.mockClear();
  Object.assign(fakeBrowser.tabs, { create: createTab });
  document.body.innerHTML = '<main id="app"></main>';
}

/** Renders the popup on `os` with the background answering `result`, then waits `ms`. */
export async function shown(
  result: ProbeResult | Promise<ProbeResult>,
  os = "linux",
  ms = 0,
): Promise<void> {
  fakeBrowser.runtime.getPlatformInfo.mockResolvedValue({ os });
  fakeBrowser.runtime.sendMessage.mockImplementation(async (message: unknown) =>
    (message as { op?: string }).op === "probe" ? result : undefined,
  );
  const { render } = await import("../src/popup/view");
  render(document.getElementById("app"));
  await vi.advanceTimersByTimeAsync(ms);
}

export const title = () => document.querySelector(".status h1")?.textContent;
export const primary = () => document.querySelector<HTMLElement>(".actions > :first-child");
export const secondary = () => document.querySelector<HTMLElement>(".actions > :nth-child(2)");
