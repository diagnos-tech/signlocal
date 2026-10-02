/**
 * A minimal `window` for the SDK: records every `postMessage`, loops it back
 * to the page's own listeners like a real browser does, and lets a test
 * deliver arbitrary `message` events (other sources, origins, garbage).
 */

export const PAGE_ORIGIN = "https://app.example";

export interface Posted {
  readonly data: unknown;
  readonly targetOrigin: string;
}

export interface FakeMessageEvent {
  readonly type: "message";
  readonly data: unknown;
  readonly source: unknown;
  readonly origin: string;
}

type Listener = (event: FakeMessageEvent) => void;

/** A window that is not the page's own: a parent, top or child frame. */
export class ForeignWindow {
  readonly received: Posted[] = [];
  postMessage(data: unknown, targetOrigin: string): void {
    this.received.push({ data, targetOrigin });
  }
}

export class FakeWindow {
  readonly posted: Posted[] = [];
  readonly parent = new ForeignWindow();
  readonly top = new ForeignWindow();
  /** Called after each post, asynchronously, like a content script. */
  onPost: ((posted: Posted) => void) | undefined;
  /** Real browsers deliver a window's own posts to its own listeners. */
  loopback = true;
  private readonly listeners = new Set<Listener>();

  constructor(readonly origin: string = PAGE_ORIGIN) {}

  addEventListener(type: string, listener: Listener): void {
    if (type === "message") this.listeners.add(listener);
  }

  removeEventListener(type: string, listener: Listener): void {
    if (type === "message") this.listeners.delete(listener);
  }

  postMessage(data: unknown, targetOrigin: string): void {
    const posted = { data: structuredClone(data), targetOrigin };
    this.posted.push(posted);
    queueMicrotask(() => {
      if (this.loopback) this.deliver({ data: posted.data });
      this.onPost?.(posted);
    });
  }

  /** Delivers a `message` event; defaults to the page's own window and origin. */
  deliver(init: { data: unknown; source?: unknown; origin?: string }): void {
    const event: FakeMessageEvent = {
      type: "message",
      data: structuredClone(init.data),
      source: "source" in init ? init.source : this,
      origin: init.origin ?? this.origin,
    };
    for (const listener of [...this.listeners]) listener(event);
  }
}
