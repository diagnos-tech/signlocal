/**
 * Handles what content scripts send: page requests to route and "this
 * document is going away" notices. Everything here came through a process the page
 * shares, so the payload is validated again and the origin is never read
 * from it.
 */

import type { Browser } from "wxt/browser";
import { browser } from "wxt/browser";
import type { PageReply } from "../generated";
import { isPageId } from "../shared/limits";
import type { RelayReply } from "../shared/runtime-messages";
import { validatePageRequest } from "../shared/validate";
import { pageSender } from "./context";
import { cancelDocument } from "./requests";
import { route } from "./router";

/** The content script's per-document token (a UUID); anything else is ignored. */
const DOCUMENT_TOKEN = /^[0-9a-f-]{36}$/;

function isDocument(value: unknown): value is string {
  return typeof value === "string" && DOCUMENT_TOKEN.test(value);
}

/**
 * Posts to the frame that asked. `document` lets the content script drop a
 * reply that arrives after the frame moved on to another page.
 */
function replier(
  tabId: number,
  frameId: number,
  document: string,
  id: string,
): (message: PageReply) => void {
  return (message) => {
    const reply: RelayReply = { kind: "websign-reply", document, id, message };
    browser.tabs.sendMessage(tabId, reply, { frameId }).catch(() => {
      // The frame is gone; nobody is left to tell.
    });
  };
}

async function relay(
  id: string,
  document: string,
  payload: unknown,
  sender: Browser.runtime.MessageSender,
): Promise<void> {
  const found = await pageSender(sender);
  if (found === null) return;
  const reply = replier(found.tabId, found.frameId, document, id);
  if (found.sender === null) {
    reply({
      type: "error",
      code: "InsecureOrigin",
      message: "SignLocal only works on https sites and on localhost",
    });
    return;
  }
  const request = validatePageRequest(payload);
  if (request === null) {
    reply({ type: "error", code: "InvalidRequest", message: "the request is malformed" });
    return;
  }
  route({ ...found.sender, document }, id, request, reply);
}

/** Handles a message from a content script; other kinds are ignored. */
export function handleContentMessage(
  message: unknown,
  sender: Browser.runtime.MessageSender,
): void {
  if (typeof message !== "object" || message === null) return;
  const { kind, document, id, message: payload } = message as Record<string, unknown>;
  const tabId = sender.tab?.id;
  if (sender.id !== browser.runtime.id || tabId === undefined || !isDocument(document)) return;
  if (kind === "websign-gone") {
    cancelDocument(tabId, sender.frameId ?? 0, document);
  } else if (kind === "websign-request" && isPageId(id)) {
    void relay(id, document, payload, sender);
  }
}
