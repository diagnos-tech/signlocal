/** A fake content script: announces itself, records page requests, replies on demand. */

import { vi } from "vitest";
import type { FakeWindow } from "./fake-window";

export interface RequestFrame {
  readonly source: "websign-page";
  readonly kind: "request";
  readonly id: string;
  readonly message: { readonly type: string; readonly [key: string]: unknown };
}

export interface ScriptOptions {
  /** Answer `discover` with an announcement (default true). */
  readonly answerDiscover?: boolean;
  readonly version?: string;
  readonly browser?: string;
  readonly protocols?: { readonly min: number; readonly max: number };
}

export class FakeScript {
  /** Replies to send right after a request arrives. */
  autoReply: (request: RequestFrame) => unknown[] | undefined = () => undefined;

  constructor(
    private readonly win: FakeWindow,
    private readonly options: ScriptOptions = {},
  ) {
    win.onPost = ({ data }) => {
      const frame = data as { source?: string; kind?: string };
      if (frame?.source !== "websign-page") return;
      if (frame.kind === "discover" && options.answerDiscover !== false) this.announce();
      if (frame.kind === "request") {
        const request = data as RequestFrame;
        for (const message of this.autoReply(request) ?? []) this.reply(request.id, message);
      }
    };
  }

  announcement(): Record<string, unknown> {
    return {
      source: "websign-extension",
      kind: "announce",
      extension: {
        version: this.options.version ?? "1.4.2",
        browser: this.options.browser ?? "chrome",
      },
      protocols: this.options.protocols ?? { min: 1, max: 1 },
    };
  }

  announce(): void {
    this.win.deliver({ data: this.announcement() });
  }

  reply(id: string, message: unknown): void {
    this.win.deliver({ data: { source: "websign-extension", kind: "message", id, message } });
  }

  get discovers(): unknown[] {
    return this.win.posted
      .map((p) => p.data)
      .filter((d) => (d as { kind?: string }).kind === "discover");
  }

  get requests(): RequestFrame[] {
    return this.win.posted
      .map((p) => p.data as RequestFrame)
      .filter((d) => d.source === "websign-page" && d.kind === "request");
  }

  requestsOfType(type: string): RequestFrame[] {
    return this.requests.filter((r) => r.message.type === type);
  }

  /** The single request of `type`; fails the test when there are none or several. */
  only(type: string): RequestFrame {
    const found = this.requestsOfType(type);
    if (found.length !== 1) throw new Error(`expected exactly one ${type}, got ${found.length}`);
    return found[0] as RequestFrame;
  }
}

/** Lets pending promise continuations and zero-delay timers run. */
export async function flush(): Promise<void> {
  await vi.advanceTimersByTimeAsync(0);
  await vi.advanceTimersByTimeAsync(0);
}
