/**
 * The page side of the page ↔ extension protocol (window.postMessage).
 *
 * Discovery: the content script announces itself at document_start and on
 * load; the SDK also posts `discover` and waits {@link DISCOVERY_TIMEOUT_MS}
 * before concluding `ExtensionMissing`.
 *
 * Only messages from this very window and origin are considered, so another
 * frame or window cannot answer in the extension's name.
 */

import { abortedError, WebSignError } from "./errors.js";
import type { ExtensionInfo, PageReply, PageRequest, ProtocolRange } from "./generated/index.js";
import { FINAL_REPLY_TYPES, parseIncoming } from "./incoming.js";

/** How long the SDK waits for the extension's announcement. */
export const DISCOVERY_TIMEOUT_MS = 1000;

/** The protocol version this SDK speaks. */
export const PROTOCOL_VERSION = 1;

const PAGE_SOURCE = "websign-page";

/** What the extension announced. */
export interface Announcement {
  readonly extension: ExtensionInfo;
  readonly protocols: ProtocolRange;
}

interface State {
  readonly win: Window;
  /** undefined: unknown yet; null: nobody answered. A later announcement replaces either. */
  announced: Announcement | null | undefined;
  discovering: Promise<Announcement | null> | undefined;
  wake: (() => void) | undefined;
  counter: number;
  readonly pending: Map<string, (reply: PageReply) => void>;
  readonly subscribers: Set<() => void>;
}

let state: State | undefined;

/**
 * The page's own origin, read at each use. Posting to it (never `"*"`) keeps
 * digests and certificates inside this origin even if the window navigated.
 */
function pageOrigin(): string {
  return (globalThis as { location?: Location }).location?.origin ?? "null";
}

/**
 * The state of the current `window`, listening from the first call on.
 * Globals are read lazily, never captured at import: importing the SDK
 * during server-side rendering (no `window`) must not throw, and a page that
 * swaps its global (tests, sandboxes) starts clean.
 */
function current(): State | undefined {
  const win = (globalThis as { window?: Window }).window;
  if (win === undefined) return undefined;
  if (state?.win === win) return state;
  const fresh: State = {
    win,
    announced: undefined,
    discovering: undefined,
    wake: undefined,
    counter: 0,
    pending: new Map(),
    subscribers: new Set(),
  };
  state = fresh;
  win.addEventListener("message", (event) => {
    if (event.source !== win || event.origin !== pageOrigin()) return;
    const message = parseIncoming(event.data);
    if (message?.kind === "announce") {
      fresh.announced = { extension: message.extension, protocols: message.protocols };
      fresh.wake?.();
      notify();
    } else if (message?.kind === "message") {
      fresh.pending.get(message.id)?.(message.reply);
    }
  });
  return fresh;
}

function post(st: State, message: Record<string, unknown>): void {
  st.win.postMessage({ source: PAGE_SOURCE, ...message }, pageOrigin());
}

/** Resolves with the announcement, or null after the timeout. Cached per page. */
export function discover(): Promise<Announcement | null> {
  const st = current();
  if (st === undefined) return Promise.resolve(null);
  if (st.announced !== undefined) return Promise.resolve(st.announced);
  st.discovering ??= new Promise((resolve) => {
    const timer = setTimeout(() => {
      st.announced ??= null;
      done();
    }, DISCOVERY_TIMEOUT_MS);
    const done = () => {
      clearTimeout(timer);
      st.wake = undefined;
      st.discovering = undefined;
      resolve(st.announced ?? null);
    };
    st.wake = done;
    post(st, { kind: "discover" });
  });
  return st.discovering;
}

/** The last announcement, without waiting; null when none was seen. */
export function lastAnnouncement(): Announcement | null {
  return current()?.announced ?? null;
}

/**
 * Sends one page request; `onReply` receives every reply with the request id
 * (`need_digest` may repeat, and the answer is a follow-up under that id)
 * until a final one. Aborting `signal` posts `cancel` and rejects
 * `done` with `Aborted` at once, without waiting for the app.
 */
export function send(
  request: PageRequest,
  onReply: (reply: PageReply, id: string) => void,
  signal?: AbortSignal,
): { readonly id: string; readonly done: Promise<void> } {
  const st = current();
  if (st === undefined)
    throw new WebSignError("ExtensionMissing", "There is no window to talk to.");
  // Counter for order, random tail so two SDK copies bundled on one page never share an id.
  const id = `p${++st.counter}.${Math.random().toString(36).slice(2, 8).padEnd(6, "0")}`;
  const done = new Promise<void>((resolve, reject) => {
    if (signal?.aborted) return reject(abortedError());
    const finish = () => {
      st.pending.delete(id);
      signal?.removeEventListener("abort", onAbort);
    };
    const onAbort = () => {
      finish();
      post(st, { kind: "request", id, message: { type: "cancel" } });
      reject(abortedError());
    };
    st.pending.set(id, (reply) => {
      try {
        onReply(reply, id);
      } catch (error) {
        finish();
        return reject(error);
      }
      if (FINAL_REPLY_TYPES.includes(reply.type)) {
        finish();
        resolve();
      }
    });
    signal?.addEventListener("abort", onAbort, { once: true });
    post(st, { kind: "request", id, message: request });
  });
  return { id, done };
}

/** Posts a follow-up (`sign.digest`, `cancel`) for the open request `id`. */
export function follow(id: string, message: PageRequest): void {
  const st = current();
  if (st !== undefined) post(st, { kind: "request", id, message });
}

/**
 * Calls `listener` whenever the extension announces itself again or a
 * request suggests the setup changed.
 */
export function subscribe(listener: () => void): () => void {
  const st = current();
  st?.subscribers.add(listener);
  return () => void st?.subscribers.delete(listener);
}

/** Tells subscribers the status may have changed. */
export function notify(): void {
  for (const listener of current()?.subscribers ?? []) listener();
}

// Listen from import time: the content script announces at document_start
// and on load, possibly before the first SDK call.
current();
