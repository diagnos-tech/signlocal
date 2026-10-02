/** A scriptable stand-in for `background/connection`'s `Connection`. */

import { vi } from "vitest";
import type { AppEnvelope, ClientEnvelope, HelloReply } from "../../src/generated";
import { helloReply } from "./protocol";

/** Records what the router sends and lets tests play the app. */
export class FakeConnection {
  readonly sent: ClientEnvelope[] = [];
  private readonly listeners = new Set<(message: AppEnvelope) => void>();
  private closeIt: (value: { heard: boolean; reason: string }) => void = () => {};
  readonly closed = new Promise<{ heard: boolean; reason: string }>((resolve) => {
    this.closeIt = resolve;
  });
  readonly send = vi.fn((message: ClientEnvelope) => {
    this.sent.push(message);
  });

  constructor(readonly hello: HelloReply = helloReply()) {}

  onMessage = (listener: (message: AppEnvelope) => void): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  /** The app answers or continues request `id`. */
  emit(id: string, body: Record<string, unknown>): void {
    const message = { v: 1, id, ...body } as unknown as AppEnvelope;
    for (const listener of [...this.listeners]) listener(message);
  }

  /** The port closes. */
  close(heard: boolean): void {
    this.closeIt({ heard, reason: "test" });
  }

  /** Everything sent with `type`. */
  sentOfType(type: string): ClientEnvelope[] {
    return this.sent.filter((m) => m.type === type);
  }
}
