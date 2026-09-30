/**
 * The native messaging port: opened on demand, `hello` first, kept open
 * while used, closed after EXTENSION_IDLE_CLOSE (60 s) without traffic.
 * Promotes the proven logic of docs/prototypes/kit/extension/native-host.js.
 *
 * Starting the app costs a process launch (and a driver load), so the port
 * stays open between signatures; a dead port is replaced on the next need,
 * and everything waiting on it hears about the death instead of hanging.
 */

import { browser } from "wxt/browser";
import type { AppEnvelope, ClientEnvelope, HelloReason, HelloReply } from "../generated";
import { NATIVE_HOST } from "../generated";
import { HELLO_TIMEOUT_MS, IDLE_CLOSE_MS, PROTOCOL_VERSION } from "../shared/limits";
import { AppError, closedCode } from "./app-error";
import { detectBrowser } from "./browser";

/** A live connection to the app. */
export interface Connection {
  readonly hello: HelloReply;
  send(message: ClientEnvelope): void;
  onMessage(listener: (message: AppEnvelope) => void): () => void;
  /** Resolves when the port closes; `heard` = the app answered at least once. */
  readonly closed: Promise<{ readonly heard: boolean; readonly reason: string }>;
}

const HELLO_ID = "hello";
/** Message types that end a request; anything else is an event or continuation. */
const FINAL_TYPES = new Set(["hello", "status", "choose.result", "sign.result", "done", "error"]);
/** Messages that belong to a request opened earlier, so they open nothing. */
const CONTINUATIONS = new Set(["sign.digest", "cancel"]);

let active: Promise<Connection> | null = null;

/** Anything that looks like an envelope; content is checked by whoever reads it. */
function isEnvelope(value: unknown): value is AppEnvelope {
  if (typeof value !== "object" || value === null) return false;
  const { id, type } = value as { id?: unknown; type?: unknown };
  return typeof id === "string" && typeof type === "string";
}

function lastErrorText(port: unknown): string {
  const fromRuntime = browser.runtime.lastError?.message;
  const fromPort = (port as { error?: { message?: string } }).error?.message;
  return fromRuntime ?? fromPort ?? "the app closed the connection";
}

function open(reason: HelloReason, forget: () => void): Promise<Connection> {
  return new Promise((resolve, reject) => {
    const listeners = new Set<(message: AppEnvelope) => void>();
    const openIds = new Set<string>();
    let heard = false;
    let ready = false;
    let ended = false;
    let idleTimer: ReturnType<typeof setTimeout> | undefined;
    let helloTimer: ReturnType<typeof setTimeout> | undefined;
    let finish: (info: { heard: boolean; reason: string }) => void = () => {};
    const closed = new Promise<{ heard: boolean; reason: string }>((done) => {
      finish = done;
    });
    let port: ReturnType<typeof browser.runtime.connectNative>;

    const end = (why: string) => {
      if (ended) return;
      ended = true;
      clearTimeout(idleTimer);
      clearTimeout(helloTimer);
      forget();
      finish({ heard, reason: why });
    };
    const hangUp = (why: string) => {
      end(why);
      try {
        port.disconnect();
      } catch {
        // Already gone.
      }
    };
    const fail = (error: AppError) => {
      hangUp(error.message);
      reject(error);
    };
    const armIdle = () => {
      clearTimeout(idleTimer);
      idleTimer = setTimeout(() => {
        if (openIds.size > 0) armIdle();
        else hangUp("idle");
      }, IDLE_CLOSE_MS);
    };

    const write = (message: ClientEnvelope) => {
      if (ended) throw new AppError(closedCode(heard), "the app closed the connection");
      if (!CONTINUATIONS.has(message.type)) openIds.add(message.id);
      port.postMessage(message);
      armIdle();
    };

    const accept = (message: AppEnvelope) => {
      if (message.type === "hello") {
        if (message.protocol !== PROTOCOL_VERSION) {
          fail(new AppError("Internal", "the app chose a protocol version we do not speak"));
          return;
        }
        ready = true;
        clearTimeout(helloTimer);
        openIds.delete(HELLO_ID);
        armIdle();
        const { app, protocol } = message;
        resolve({
          hello: { app, protocol },
          send: write,
          closed,
          onMessage(listener) {
            listeners.add(listener);
            return () => listeners.delete(listener);
          },
        });
        return;
      }
      if (message.type === "error") {
        const code = message.code === "ClientOutdated" ? "ExtensionOutdated" : message.code;
        fail(new AppError(code, message.message, message.details));
        return;
      }
      fail(new AppError("Internal", "the app's first message was not hello"));
    };

    const dispatch = (message: AppEnvelope) => {
      if (FINAL_TYPES.has(message.type)) openIds.delete(message.id);
      armIdle();
      for (const listener of [...listeners]) {
        try {
          listener(message);
        } catch {
          // One listener's bug must not starve the others of their replies.
        }
      }
    };

    try {
      port = browser.runtime.connectNative(NATIVE_HOST);
    } catch (error) {
      reject(new AppError("AppMissing", error instanceof Error ? error.message : "no host"));
      return;
    }
    port.onMessage.addListener((message: unknown) => {
      heard = true;
      if (ended || !isEnvelope(message)) return;
      if (ready) dispatch(message);
      else accept(message);
    });
    port.onDisconnect.addListener(() => {
      const why = lastErrorText(port);
      const wasReady = ready;
      end(why);
      if (!wasReady) reject(new AppError(closedCode(heard), why));
    });
    helloTimer = setTimeout(
      () => fail(new AppError("Internal", "the app did not respond")),
      HELLO_TIMEOUT_MS,
    );
    void detectBrowser().then((identity) => {
      if (ended) return;
      port.postMessage({
        v: PROTOCOL_VERSION,
        id: HELLO_ID,
        type: "hello",
        client: { name: "websign-extension", version: browser.runtime.getManifest().version },
        protocols: { min: PROTOCOL_VERSION, max: PROTOCOL_VERSION },
        browser: { ...identity, reason },
      } satisfies ClientEnvelope);
    });
  });
}

/**
 * The current connection, opening it (and saying hello) if needed. `reason`
 * only matters for the connection this call opens.
 */
export function connect(reason: HelloReason = "page"): Promise<Connection> {
  if (active === null) {
    const attempt: Promise<Connection> = open(reason, () => {
      if (active === attempt) active = null;
    });
    active = attempt;
    attempt.catch(() => {
      if (active === attempt) active = null;
    });
  }
  return active;
}
