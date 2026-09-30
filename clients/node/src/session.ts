/**
 * One `websign connect` child process and the request bookkeeping on top of
 * its framed stdin/stdout. Everything that can go wrong with the process
 * (missing binary, early exit, garbage, silence) ends here as a
 * {@link WebSignError}, so the API layer only sees replies and failures.
 */

import { type ChildProcess, spawn } from "node:child_process";
import { fromWire, WebSignError } from "./errors.js";
import { encodeFrame, FrameDecoder } from "./framing.js";
import type { AppEnvelope } from "./generated/index.js";
import { type ClientIdentity, helloFields, negotiatedVersion, PROTOCOL } from "./hello.js";
import { holdEventLoop, killOnExit } from "./lifetime.js";

export interface Timings {
  /** How long the app has to answer `hello`. */
  readonly helloMs: number;
  /** How long `close()` waits for the app to exit before killing it. */
  readonly closeMs: number;
}

export const DEFAULT_TIMINGS: Timings = { helloMs: 10_000, closeMs: 5_000 };

/** A request in flight; the app answers with any number of events and one final message. */
export interface Exchange {
  readonly id: string;
  /** The final message; rejects with the app's `error` or a local failure. */
  readonly result: Promise<AppEnvelope>;
  /** Continuation on the same id (`sign.digest`, `cancel`). Ignored once settled. */
  send(type: string, fields?: Record<string, unknown>): void;
  /** Ends the exchange locally with `error` and tells the app to `cancel`. */
  fail(error: WebSignError): void;
}

interface Pending {
  readonly onEvent: ((event: AppEnvelope) => void) | undefined;
  readonly resolve: (message: AppEnvelope) => void;
  readonly reject: (error: WebSignError) => void;
}

export class Session {
  private readonly decoder = new FrameDecoder();
  private readonly pending = new Map<string, Pending>();
  private nextId = 1;
  private version = PROTOCOL;
  private ready = false;
  private exited = false;
  private ending: WebSignError | undefined;
  private closing: Promise<void> | undefined;
  private readonly forgetExit: () => void;

  private constructor(
    private readonly child: ChildProcess,
    private readonly timings: Timings,
  ) {
    child.stdout?.on("data", (chunk: Buffer) => this.receive(chunk));
    child.stdin?.on("error", () => {});
    child.on("error", (error) => {
      this.end(new WebSignError("AppMissing", `could not start the app: ${error.message}`), true);
    });
    child.on("close", () => {
      this.exited = true;
      this.forgetExit();
      this.end(
        this.ready
          ? new WebSignError("Internal", "the app exited")
          : new WebSignError("AppMissing", "the app exited before answering hello"),
        false,
      );
    });
    this.forgetExit = killOnExit(child);
  }

  /** Spawns `executable connect` and completes the `hello` negotiation. */
  static async open(
    executable: string,
    client: ClientIdentity,
    timings: Timings = DEFAULT_TIMINGS,
  ): Promise<Session> {
    const child = spawn(executable, ["connect"], {
      stdio: ["pipe", "pipe", "ignore"],
      windowsHide: true,
    });
    const session = new Session(child, timings);
    const timer = setTimeout(() => {
      session.end(new WebSignError("Timeout", "the app did not answer hello in time"), true);
    }, timings.helloMs);
    try {
      const reply = await session.request("hello", helloFields(client)).result;
      session.version = negotiatedVersion(reply);
      session.ready = true;
      return session;
    } catch (error) {
      session.end(
        error instanceof WebSignError ? error : new WebSignError("Internal", "hello failed"),
        true,
      );
      throw error;
    } finally {
      clearTimeout(timer);
    }
  }

  /** Sends a request; `onEvent` receives the non-final messages of that id. */
  request(
    type: string,
    fields: Record<string, unknown> = {},
    onEvent?: (event: AppEnvelope) => void,
  ): Exchange {
    const id = `n${this.nextId++}`;
    let settle: Pending | undefined;
    const result = new Promise<AppEnvelope>((resolve, reject) => {
      settle = { onEvent, resolve, reject };
    });
    const exchange: Exchange = {
      id,
      result,
      send: (nextType, nextFields) => {
        if (this.pending.has(id)) this.write({ id, type: nextType, ...nextFields });
      },
      fail: (error) => {
        const open = this.pending.get(id);
        if (!open) return;
        this.pending.delete(id);
        this.holdWhileBusy();
        this.write({ id, type: "cancel" });
        open.reject(error);
      },
    };
    if (this.ending) {
      settle?.reject(this.ending);
      return exchange;
    }
    if (settle) this.pending.set(id, settle);
    this.holdWhileBusy();
    this.write({ id, type, ...fields });
    return exchange;
  }

  /** Ends stdin so the app exits; kills it if it does not within the close timeout. */
  close(): Promise<void> {
    this.closing ??= this.shutdown();
    return this.closing;
  }

  private async shutdown(): Promise<void> {
    if (this.exited) return;
    this.end(new WebSignError("Internal", "the connection was closed"), false);
    holdEventLoop(this.child, true);
    const exit = new Promise<void>((resolve) => this.child.once("close", () => resolve()));
    this.child.stdin?.end();
    const timer = setTimeout(() => this.child.kill("SIGKILL"), this.timings.closeMs);
    try {
      await exit;
    } finally {
      clearTimeout(timer);
    }
  }

  private write(message: Record<string, unknown>): void {
    if (this.exited || !this.child.stdin?.writable) return;
    try {
      this.child.stdin.write(encodeFrame({ v: this.version, ...message }));
    } catch (error) {
      if (error instanceof WebSignError && typeof message.id === "string") {
        this.pending.get(message.id)?.reject(error);
        this.pending.delete(message.id);
        this.holdWhileBusy();
        return;
      }
      throw error;
    }
  }

  private receive(chunk: Buffer): void {
    let messages: unknown[];
    try {
      messages = this.decoder.push(chunk);
    } catch (error) {
      this.end(error as WebSignError, true);
      return;
    }
    for (const message of messages) {
      if (!this.route(message)) {
        this.end(
          new WebSignError("Internal", "the app sent a message without an id or type"),
          true,
        );
        return;
      }
    }
  }

  /** False when the message is not an envelope at all (a protocol violation). */
  private route(message: unknown): boolean {
    if (typeof message !== "object" || message === null) return false;
    const envelope = message as Record<string, unknown>;
    if (typeof envelope.id !== "string" || typeof envelope.type !== "string") return false;
    const open = this.pending.get(envelope.id);
    if (!open) return true;
    if (envelope.type === "sign.need_digest" && open.onEvent) {
      open.onEvent(envelope as unknown as AppEnvelope);
      return true;
    }
    if (!this.ready && envelope.type === "error" && envelope.v !== this.version) {
      this.end(
        new WebSignError("Internal", "the app refused hello at another protocol version"),
        true,
      );
      return true;
    }
    this.pending.delete(envelope.id);
    this.holdWhileBusy();
    if (envelope.type === "error") open.reject(fromWire(envelope));
    else open.resolve(envelope as unknown as AppEnvelope);
    return true;
  }

  private holdWhileBusy(): void {
    if (this.closing === undefined) holdEventLoop(this.child, this.pending.size > 0);
  }

  /** Fails everything in flight; optionally kills the child (it is misbehaving or unwanted). */
  private end(error: WebSignError, kill: boolean): void {
    this.ending ??= error;
    for (const open of this.pending.values()) open.reject(this.ending);
    this.pending.clear();
    this.holdWhileBusy();
    if (kill && !this.exited) this.child.kill("SIGKILL");
  }
}
