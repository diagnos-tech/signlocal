/**
 * Page ↔ background relay: accepts only same-window, same-origin messages
 * with source "websign-page", rebuilds the request field by field, and posts
 * replies back with source "websign-extension" to the page's own origin.
 * The background trusts none of this (it validates again and reads the
 * origin from the browser); this layer keeps page inventions from travelling.
 */

import type { Browser } from "wxt/browser";
import { browser } from "wxt/browser";
import type { ExtensionToPage, PageReply } from "../generated";
import { EXTENSION_SOURCE, isPageId, PAGE_SOURCE } from "../shared/limits";
import type { RelayGone, RelayRequest } from "../shared/runtime-messages";
import { validatePageRequest } from "../shared/validate";
import { announce } from "./announce";

/**
 * Tells this document's requests apart from those of the next document in
 * the same frame, so a late "gone" never cancels the page that replaced it.
 */
const DOCUMENT = crypto.randomUUID();

function isObject(value: unknown): value is object {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function postToPage(id: string, message: PageReply): void {
  const envelope: ExtensionToPage = { kind: "message", source: EXTENSION_SOURCE, id, message };
  window.postMessage(envelope, location.origin);
}

function forward(id: string, payload: unknown): void {
  const message = validatePageRequest(payload);
  if (message === null) {
    postToPage(id, { type: "error", code: "InvalidRequest", message: "the request is malformed" });
    return;
  }
  const request: RelayRequest = { kind: "websign-request", document: DOCUMENT, id, message };
  browser.runtime.sendMessage(request).catch(() => {
    postToPage(id, { type: "error", code: "Internal", message: "the extension is not available" });
  });
}

function onPageMessage(event: MessageEvent): void {
  if (event.source !== window || event.origin !== location.origin) return;
  const data: unknown = event.data;
  if (!isObject(data)) return;
  const { source, kind, id, message } = data as Record<string, unknown>;
  if (source !== PAGE_SOURCE) return;
  if (kind === "discover") announce();
  else if (kind === "request" && isPageId(id)) forward(id, message);
}

/** Replies from the background, and the top frame's origin when it asks (SPEC §2.3). */
function onExtensionMessage(
  message: unknown,
  sender: Browser.runtime.MessageSender,
  respond: (origin: string) => void,
): boolean {
  if (sender.id !== browser.runtime.id || typeof message !== "object" || message === null) {
    return false;
  }
  const { kind, document, id, message: reply } = message as Record<string, unknown>;
  if (kind === "websign-reply" && document === DOCUMENT && isPageId(id) && isObject(reply)) {
    postToPage(id, reply as PageReply);
  } else if (kind === "websign-origin" && window === window.top) {
    respond(location.origin);
  }
  return false;
}

/** Starts relaying for this frame. */
export function startRelay(): void {
  window.addEventListener("message", onPageMessage);
  browser.runtime.onMessage.addListener(onExtensionMessage);
  // A document that goes away (reload, navigation, close, back-forward cache)
  // must not leave its window open in the app.
  window.addEventListener("pagehide", () => {
    const gone: RelayGone = { kind: "websign-gone", document: DOCUMENT };
    browser.runtime.sendMessage(gone).catch(() => {});
  });
}
