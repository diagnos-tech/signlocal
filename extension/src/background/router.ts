/**
 * Routes page requests to the app and replies back to the right frame.
 * Page ids are only unique per page, so each (tab, frame, page id) gets a
 * connection-wide native id; closing a tab or navigating cancels its open
 * requests (the app shows "{site} cancelled the request").
 */

import type { PageReply, PageRequest, WebContext } from "../generated";

/** Where a page request came from, as the browser reported it. */
export interface PageSender {
  readonly tabId: number;
  readonly frameId: number;
  readonly context: WebContext;
}

/** Forwards `request`; `reply` receives every message for it until the final one. */
export function route(
  sender: PageSender,
  pageId: string,
  request: PageRequest,
  reply: (message: PageReply) => void,
): void {
  void sender;
  void pageId;
  void request;
  void reply;
  throw new Error("unimplemented: SPEC.md §2.2");
}
