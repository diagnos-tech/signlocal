/**
 * The page side of the page ↔ extension protocol (window.postMessage).
 *
 * Discovery: the content script announces itself at document_start and on
 * load; the SDK also posts `discover` and waits {@link DISCOVERY_TIMEOUT_MS}
 * before concluding `ExtensionMissing`.
 */

import type { ExtensionInfo, PageReply, PageRequest, ProtocolRange } from "./generated";

/** How long the SDK waits for the extension's announcement. */
export const DISCOVERY_TIMEOUT_MS = 1000;

/** What the extension announced. */
export interface Announcement {
  readonly extension: ExtensionInfo;
  readonly protocols: ProtocolRange;
}

/** Resolves with the announcement, or null after the timeout. Cached per page. */
export function discover(): Promise<Announcement | null> {
  throw new Error("unimplemented: SPEC.md §2");
}

/** Sends one page request; `onReply` receives every reply (need_digest may repeat) until a final one. */
export function send(
  request: PageRequest,
  onReply: (reply: PageReply) => void,
  signal?: AbortSignal,
): { readonly id: string; readonly done: Promise<void> } {
  void request;
  void onReply;
  void signal;
  throw new Error("unimplemented: SPEC.md §2");
}
