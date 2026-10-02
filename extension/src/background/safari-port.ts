/**
 * A native port for Safari, built from its app extension's relay sessions
 * (`safari/SPEC.md` §3). Safari gives the extension no stdio connection:
 * each `runtime.sendNativeMessage` is one question with one answer, and the
 * appex can never speak first. So this port opens a session (one host
 * process, as `connectNative` starts elsewhere), sends each message into it
 * in order, and keeps exactly one long poll outstanding for what the host
 * says, until either side closes it.
 */

import { browser } from "wxt/browser";
import type { ErrorCode } from "../generated";
import { NATIVE_HOST } from "../generated";
import type { NativePort } from "./native-port";
import {
  CALL_TIMEOUT_MS,
  POLL_WAIT_MS,
  parseRelayReply,
  RELAY_VERSION,
  REPLY_GRACE_MS,
  type RelayRequest,
  relayErrorCode,
} from "./safari-relay";

/** `sendNativeMessage` as Safari implements it; tests pass a fake. */
export type SendNative = (message: RelayRequest) => Promise<unknown>;

/**
 * Safari ignores the application identifier and always delivers to the
 * containing app's extension; the argument exists for the API's shape.
 */
const safariSend: SendNative = (message) => browser.runtime.sendNativeMessage(NATIVE_HOST, message);

/** A poll answered empty this fast, with nothing sent meanwhile, is a relay not parking polls. */
const MIN_BACKOFF_MS = 250;
const MAX_BACKOFF_MS = POLL_WAIT_MS;

class PortEvent<A extends unknown[]> {
  private readonly listeners: Array<(...args: A) => void> = [];
  addListener(listener: (...args: A) => void): void {
    this.listeners.push(listener);
  }
  emit(...args: A): void {
    for (const listener of [...this.listeners]) listener(...args);
  }
}

class Timeout extends Error {}

export class SafariPort implements NativePort {
  readonly onMessage = new PortEvent<[unknown]>();
  readonly onDisconnect = new PortEvent<[]>();
  error: { message: string; code?: ErrorCode } | null = null;
  private session: string | null = null;
  private ended = false;
  private readonly outbox: object[] = [];
  private sending = false;
  private polling = false;
  private sentDuringPoll = false;
  private backoff = 0;
  private pollTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(private readonly send: SendNative = safariSend) {
    void this.open();
  }

  postMessage(message: object): void {
    if (this.ended) throw new Error("Attempting to use a disconnected port object");
    this.outbox.push(message);
    void this.pump();
  }

  disconnect(): void {
    if (this.ended) return;
    this.stop();
    this.closeSession();
  }

  private async open(): Promise<void> {
    const reply = await this.exchange({ relay: RELAY_VERSION, op: "open" }, CALL_TIMEOUT_MS);
    if (reply === null) return;
    void this.pump();
    this.schedulePoll(0);
  }

  /** Sends queued messages one at a time, so the host reads them in order. */
  private async pump(): Promise<void> {
    while (!this.sending && !this.ended && this.session !== null && this.outbox.length > 0) {
      const message = this.outbox.shift() as object;
      this.sending = true;
      this.sentDuringPoll = true;
      const request = { relay: RELAY_VERSION, op: "send", session: this.session, message } as const;
      const reply = await this.exchange(request, CALL_TIMEOUT_MS);
      this.sending = false;
      if (reply === null) return;
    }
  }

  private schedulePoll(delay: number): void {
    if (this.ended) return;
    clearTimeout(this.pollTimer);
    this.pollTimer = setTimeout(() => void this.poll(), delay);
  }

  private async poll(): Promise<void> {
    if (this.ended || this.polling || this.session === null) return;
    this.polling = true;
    this.sentDuringPoll = false;
    const started = Date.now();
    const request = {
      relay: RELAY_VERSION,
      op: "poll",
      session: this.session,
      wait: POLL_WAIT_MS,
    } as const;
    const delivered = await this.exchange(request, POLL_WAIT_MS + REPLY_GRACE_MS);
    this.polling = false;
    if (delivered === null) return;
    // A newer request answers a parked poll at once, empty: poll again at
    // once. An early empty answer with nothing sent means the relay did not
    // park the poll; backing off keeps that from becoming a busy loop.
    const early = Date.now() - started < POLL_WAIT_MS / 2;
    const idleSpin = delivered === 0 && early && !this.sentDuringPoll;
    this.backoff = idleSpin
      ? Math.min(Math.max(this.backoff * 2, MIN_BACKOFF_MS), MAX_BACKOFF_MS)
      : 0;
    this.schedulePoll(this.backoff);
  }

  /**
   * One request and its reply: delivers the host's messages and ends the
   * port on any failure. Returns how many messages arrived, or null when
   * the port is (now) closed.
   */
  private async exchange(request: RelayRequest, timeoutMs: number): Promise<number | null> {
    let raw: unknown;
    try {
      raw = await this.call(request, timeoutMs);
    } catch (error) {
      const timedOut = error instanceof Timeout;
      this.fail(timedOut ? "the Safari extension did not answer" : messageOf(error));
      return null;
    }
    const reply = parseRelayReply(raw);
    if (this.ended) {
      // Disconnected while `open` was in flight: end the host it started.
      if (request.op === "open" && reply?.kind === "session" && reply.open) {
        this.session = reply.session;
        this.closeSession();
      }
      return null;
    }
    const foreign =
      reply?.kind === "session" && this.session !== null && reply.session !== this.session;
    if (reply === null || foreign) {
      this.fail("the Safari extension sent a malformed reply", "Internal");
      return null;
    }
    if (reply.kind === "error") {
      this.fail(`the Safari extension refused: ${reply.code}`, relayErrorCode(reply.code));
      return null;
    }
    this.session = reply.session;
    for (const message of reply.messages) {
      if (this.ended) return null;
      this.onMessage.emit(message);
    }
    if (this.ended) return null;
    if (!reply.open) {
      this.end("the app closed the connection");
      return null;
    }
    return reply.messages.length;
  }

  private async call(request: RelayRequest, timeoutMs: number): Promise<unknown> {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const timeout = new Promise<never>((_, reject) => {
      timer = setTimeout(() => reject(new Timeout()), timeoutMs);
    });
    try {
      return await Promise.race([this.send(request), timeout]);
    } finally {
      clearTimeout(timer);
    }
  }

  /** Ends the port and releases the host, which may still be running. */
  private fail(message: string, code: ErrorCode | null = null): void {
    if (this.ended) return;
    this.end(message, code);
    this.closeSession();
  }

  private end(message: string, code: ErrorCode | null = null): void {
    if (this.ended) return;
    this.stop();
    this.error = code === null ? { message } : { message, code };
    this.onDisconnect.emit();
  }

  private stop(): void {
    this.ended = true;
    this.outbox.length = 0;
    clearTimeout(this.pollTimer);
  }

  private closeSession(): void {
    const session = this.session;
    if (session === null) return;
    this.session = null;
    this.send({ relay: RELAY_VERSION, op: "close", session }).catch(() => {
      // The appex ends sessions nobody touches; nothing else to do.
    });
  }
}

function messageOf(error: unknown): string {
  return error instanceof Error && error.message !== "" ? error.message : "no host";
}
