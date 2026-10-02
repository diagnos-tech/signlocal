/**
 * Who is asking, from the browser's MessageSender. The page and its content
 * script are untrusted; only `tab`, `frameId` and `url` — which the browser
 * fills in — are believed (SPEC §2.3).
 */

import type { Browser } from "wxt/browser";
import { browser } from "wxt/browser";
import type { OriginQuery } from "../shared/runtime-messages";
import { webContext } from "./origin";
import type { PageSender } from "./router";

/** True for messages sent by this extension's own content scripts, popup or pages. */
export function isOwnSender(sender: Browser.runtime.MessageSender): boolean {
  return sender.id === browser.runtime.id;
}

/**
 * The top frame's URL. `sender.tab.url` needs host permission for the tab;
 * when a browser withholds it, the top frame's own content script is asked
 * — never the requesting frame, which could lie about its parent.
 */
async function topUrl(
  sender: Browser.runtime.MessageSender,
  tabId: number,
): Promise<string | undefined> {
  if (sender.frameId === 0) return sender.url;
  if (sender.tab?.url) return sender.tab.url;
  try {
    const query: OriginQuery = { kind: "websign-origin" };
    const origin: unknown = await browser.tabs.sendMessage(tabId, query, { frameId: 0 });
    return typeof origin === "string" ? origin : undefined;
  } catch {
    return undefined;
  }
}

/** The sender's tab, frame and secure web context; null for non-tab senders or insecure pages. */
export async function pageSender(
  sender: Browser.runtime.MessageSender,
): Promise<{ tabId: number; frameId: number; sender: PageSender | null } | null> {
  const tabId = sender.tab?.id;
  if (!isOwnSender(sender) || tabId === undefined) return null;
  const frameId = sender.frameId ?? 0;
  const context = webContext(sender.url, await topUrl(sender, tabId));
  return { tabId, frameId, sender: context && { tabId, frameId, context } };
}
