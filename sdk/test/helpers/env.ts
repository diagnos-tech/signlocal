/**
 * Loads a fresh copy of the SDK against a fake window. The SDK listens on
 * import, so every test gets its own module graph and its own window.
 */

import { afterEach, beforeEach, vi } from "vitest";
import { FakeScript, type ScriptOptions } from "./fake-script";
import { FakeWindow } from "./fake-window";

export interface Env {
  readonly win: FakeWindow;
  readonly script: FakeScript;
  readonly sdk: typeof import("../../src/index");
  readonly channel: typeof import("../../src/channel");
  /** Global names that appeared while importing the SDK. */
  readonly importedGlobals: readonly string[];
}

export interface LoadOptions extends ScriptOptions {
  readonly origin?: string;
  readonly userAgent?: string;
}

/** Fake timers on for every test of the calling file; everything restored after. */
export function useFakeEnvironment(): void {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.useRealTimers();
    vi.restoreAllMocks();
  });
}

export async function loadSdk(options: LoadOptions = {}): Promise<Env> {
  vi.resetModules();
  const win = new FakeWindow(options.origin);
  const script = new FakeScript(win, options);
  vi.stubGlobal("window", win);
  vi.stubGlobal("self", win);
  vi.stubGlobal("location", { origin: win.origin, href: `${win.origin}/` });
  vi.stubGlobal("addEventListener", win.addEventListener.bind(win));
  vi.stubGlobal("removeEventListener", win.removeEventListener.bind(win));
  vi.stubGlobal("postMessage", win.postMessage.bind(win));
  vi.stubGlobal("navigator", {
    userAgent: options.userAgent ?? "Mozilla/5.0 Chrome/129.0 Safari/537.36",
  });
  const before = new Set(Object.getOwnPropertyNames(globalThis));
  const sdk = await import("../../src/index");
  const channel = await import("../../src/channel");
  const importedGlobals = Object.getOwnPropertyNames(globalThis).filter((k) => !before.has(k));
  return { win, script, sdk, channel, importedGlobals };
}

/** A settled outcome, so tests can assert on rejections without unhandled-rejection noise. */
export type Outcome<T> = { ok: true; value: T } | { ok: false; error: unknown };

export function settle<T>(promise: Promise<T>): { readonly outcome: () => Outcome<T> | undefined } {
  let result: Outcome<T> | undefined;
  promise.then(
    (value) => {
      result = { ok: true, value };
    },
    (error: unknown) => {
      result = { ok: false, error };
    },
  );
  return { outcome: () => result };
}
