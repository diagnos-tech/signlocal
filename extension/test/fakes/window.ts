/** A minimal `window` for the content script (the suite runs without a DOM). */

import { vi } from "vitest";

/** What a `message` event carries, as far as the relay reads it. */
export interface FakeMessageEvent {
  source: unknown;
  origin: string;
  data: unknown;
}

/** The page's window, with recorded `postMessage` calls. */
export class FakeWindow {
  readonly location: { origin: string };
  readonly posted: Array<{ message: Record<string, unknown>; targetOrigin: string }> = [];
  private readonly handlers = new Map<string, Array<(event: FakeMessageEvent) => void>>();
  addEventListener = vi.fn((type: string, handler: (event: FakeMessageEvent) => void) => {
    this.handlers.set(type, [...(this.handlers.get(type) ?? []), handler]);
  });
  postMessage = vi.fn((message: Record<string, unknown>, targetOrigin: string) => {
    this.posted.push({ message, targetOrigin });
  });

  constructor(origin: string) {
    this.location = { origin };
  }

  /** Delivers a `message` event; defaults to a same-window, same-origin one. */
  dispatchMessage(data: unknown, overrides: Partial<FakeMessageEvent> = {}): void {
    const event = { source: this, origin: this.location.origin, data, ...overrides };
    for (const handler of this.handlers.get("message") ?? []) handler(event);
  }

  /** Fires a bare event such as `load`. */
  dispatch(type: string): void {
    for (const handler of this.handlers.get(type) ?? []) handler({} as FakeMessageEvent);
  }

  /** Messages the extension posted to the page. */
  fromExtension(): Array<Record<string, unknown>> {
    return this.posted.map((p) => p.message).filter((m) => m.source === "websign-extension");
  }
}

/** Installs a fresh window as the globals `window` and `location`. */
export function installWindow(origin = "https://app.example.com"): FakeWindow {
  const win = new FakeWindow(origin);
  Object.assign(globalThis, { window: win, location: win.location });
  return win;
}
