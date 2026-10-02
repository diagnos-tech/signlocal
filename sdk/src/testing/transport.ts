/**
 * How the fake talks to the SDK: `message` events on the page's own window,
 * exactly where the real content script posts them (same window as source,
 * the page's origin), so the SDK's origin and source checks run unchanged.
 */

import type { ExtensionToPage, PageReply } from "../generated/index.js";

export interface Transport {
  announce(): void;
  reply(id: string, message: PageReply): void;
  stop(): void;
}

/** A page frame the fake received, not yet validated. */
export type PageFrame = {
  readonly kind?: unknown;
  readonly id?: unknown;
  readonly message?: unknown;
};

export interface TransportOptions {
  readonly window: Window;
  readonly latencyMs: number;
  readonly announcement: () => ExtensionToPage;
  readonly onFrame: (frame: PageFrame) => void;
  /** Called once if another extension (a real one) answers on this page too. */
  readonly onForeignExtension: () => void;
}

/**
 * Dispatches the event itself rather than calling `postMessage`: jsdom and
 * some other test DOMs deliver posted messages without `source`/`origin`,
 * which the SDK rightly drops.
 */
function dispatch(win: Window, data: ExtensionToPage): void {
  const init = { data, origin: win.location?.origin ?? "null", source: win };
  let event: Event;
  try {
    const Message = (win as unknown as { MessageEvent?: typeof MessageEvent }).MessageEvent;
    event = new (Message ?? MessageEvent)("message", init);
  } catch {
    // Test DOMs whose MessageEvent refuses a window as `source`.
    event = Object.assign(new Event("message"), init);
  }
  win.dispatchEvent(event);
}

export function createTransport(options: TransportOptions): Transport {
  const { window: win, latencyMs } = options;
  const own = new WeakSet<object>();
  let stopped = false;
  let warned = false;

  const send = (data: ExtensionToPage) => {
    const run = () => {
      if (stopped) return;
      own.add(data);
      dispatch(win, data);
    };
    if (latencyMs > 0) setTimeout(run, latencyMs);
    else queueMicrotask(run);
  };

  const listener = (event: MessageEvent) => {
    const data = event.data as { source?: unknown; kind?: unknown } | null;
    if (typeof data !== "object" || data === null) return;
    if (data.source === "websign-page") options.onFrame(data as PageFrame);
    else if (data.source === "websign-extension" && !own.has(data) && !warned) {
      warned = true;
      options.onForeignExtension();
    }
  };
  win.addEventListener("message", listener);

  return {
    announce: () => send(options.announcement()),
    reply: (id, message) => send({ source: "websign-extension", kind: "message", id, message }),
    stop: () => {
      stopped = true;
      win.removeEventListener("message", listener);
    },
  };
}
