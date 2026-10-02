/**
 * A browser-like window for the testing entry: an EventTarget whose
 * postMessage delivers a `message` event with this window as `source` and
 * its origin, one task later, like browsers do. Both the SDK and the fake
 * are loaded fresh against it, with real timers (the fake signs with WebCrypto).
 */

import { afterEach, vi } from "vitest";

export class DomWindow extends EventTarget {
  readonly location: { origin: string; protocol: string; hostname: string; href: string };

  constructor(url = "http://localhost:5173/") {
    super();
    const { origin, protocol, hostname, href } = new URL(url);
    this.location = { origin, protocol, hostname, href };
  }

  postMessage(data: unknown, targetOrigin: string): void {
    if (targetOrigin !== "*" && targetOrigin !== this.location.origin) return;
    const cloned = structuredClone(data);
    setTimeout(() => {
      const event = Object.assign(new Event("message"), {
        data: cloned,
        origin: this.location.origin,
        source: this,
      });
      this.dispatchEvent(event);
    }, 0);
  }
}

export interface Loaded {
  readonly win: DomWindow;
  readonly sdk: typeof import("../../src/index");
  readonly testing: typeof import("../../src/testing/index");
  readonly warn: ReturnType<typeof vi.spyOn>;
}

/** Fresh SDK + fake on a fresh window at `url`. */
export async function load(url?: string): Promise<Loaded> {
  vi.resetModules();
  const win = new DomWindow(url);
  vi.stubGlobal("window", win);
  vi.stubGlobal("location", win.location);
  const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
  const sdk = await import("../../src/index");
  const testing = await import("../../src/testing/index");
  return { win, sdk, testing, warn };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});
