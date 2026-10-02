/**
 * The table of page requests the app has not finished answering, keyed by
 * native id `"<tabId>.<frameId>.<pageId>"`. The tab and frame come from the
 * browser's MessageSender, so a page can only ever name, continue or cancel
 * its own requests: another tab's ids are outside its reach by construction.
 */

import type {
  AppInfo,
  ClientEnvelope,
  ErrorCode,
  HashName,
  PageReply,
  PageRequest,
} from "../generated";
import { PROTOCOL_VERSION } from "../shared/limits";
import { digestFitsHash } from "../shared/validate";
import { AppError } from "./app-error";
import type { Connection } from "./connection";

/** One open request. */
export interface Entry {
  readonly tabId: number;
  readonly frameId: number;
  readonly topOrigin: string;
  readonly document: string | undefined;
  readonly reply: (message: PageReply) => void;
  /** The hash `sign.begin` declared; every digest for it must have its exact length. */
  readonly hash?: HashName;
  /** Null while the connection is still opening. */
  conn: Connection | null;
  /** Set for `status`, which is answered even when the app stays silent. */
  app?: AppInfo;
  timer?: ReturnType<typeof setTimeout>;
}

/** Open requests by native id. */
export const entries = new Map<string, Entry>();

/** The native id of a page request; tab and frame ids never contain dots. */
export function nativeId(tabId: number, frameId: number, pageId: string): string {
  return `${tabId}.${frameId}.${pageId}`;
}

/** Forgets a request that got its final answer (or never will). */
export function finish(id: string, entry: Entry): void {
  clearTimeout(entry.timer);
  entries.delete(id);
}

/** An `error` page reply. */
export function failure(code: ErrorCode, message: string): PageReply {
  return { type: "error", code, message };
}

/** The page's view of a failure to reach the app. */
export function toError(error: unknown): PageReply {
  if (!(error instanceof AppError)) {
    return failure("Internal", "unexpected failure while contacting the app");
  }
  const { code, message, details } = error;
  return details ? { type: "error", code, message, details } : { type: "error", code, message };
}

/** Whether `request` is a digest that cannot belong to `entry`'s hash. */
function wrongDigest(entry: Entry, request: PageRequest): boolean {
  if (request.type !== "sign.digest") return false;
  return entry.hash === undefined || !digestFitsHash(entry.hash, request.digest);
}

/**
 * Sends a continuation (`sign.digest`, `cancel`) for a request that is still
 * open. A digest that does not fit the request's hash ends the request
 * here (the app would refuse it too, but nothing malformed travels further).
 */
export function continuation(id: string, request: PageRequest): void {
  const entry = entries.get(id);
  if (entry === undefined) return;
  if (entry.conn === null) {
    // Still connecting: nothing reached the app yet, so there is nothing to continue.
    finish(id, entry);
    return;
  }
  if (wrongDigest(entry, request)) {
    continuation(id, { type: "cancel" });
    finish(id, entry);
    entry.reply(failure("InvalidRequest", "the digest length does not match the request's hash"));
    return;
  }
  try {
    entry.conn.send({ ...request, v: PROTOCOL_VERSION, id } as ClientEnvelope);
  } catch {
    // The connection is closing; the router's close handler answers the page.
  }
}

/** Sends `cancel` for (and forgets) every open request `matches` selects. */
function cancelWhere(matches: (entry: Entry) => boolean): void {
  for (const [id, entry] of [...entries]) {
    if (!matches(entry)) continue;
    continuation(id, { type: "cancel" });
    finish(id, entry);
  }
}

/** The tab closed: its pages are gone. */
export function cancelTab(tabId: number): void {
  cancelWhere((entry) => entry.tabId === tabId);
}

/** The tab's top frame is loading `origin`: requests made under another top origin end. */
export function cancelNavigated(tabId: number, origin: string): void {
  cancelWhere((entry) => entry.tabId === tabId && entry.topOrigin !== origin);
}

/** A document went away (`pagehide`): only its own requests end, not its successor's. */
export function cancelDocument(tabId: number, frameId: number, document: string): void {
  cancelWhere(
    (entry) => entry.tabId === tabId && entry.frameId === frameId && entry.document === document,
  );
}
