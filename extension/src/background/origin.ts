/**
 * The requesting origins, from the browser's MessageSender (never from the
 * payload). Refuses non-secure contexts with InsecureOrigin before anything
 * reaches the app: http other than localhost, file:, data:, extensions.
 */

import type { WebContext } from "../generated";
import { MAX_ORIGIN_LEN } from "../shared/limits.gen";

/**
 * https anywhere; http only for names and addresses that never leave the
 * machine and that the content script's matches reach (entrypoints/content.ts):
 * match patterns cannot name the rest of 127.0.0.0/8, so accepting it here
 * would only pretend to support it.
 */
function originIfSecure(text: string | undefined): string | null {
  if (text === undefined) return null;
  let url: URL;
  try {
    url = new URL(text);
  } catch {
    return null;
  }
  const host = url.hostname;
  const local =
    host === "localhost" || host.endsWith(".localhost") || host === "127.0.0.1" || host === "[::1]";
  const secure = url.protocol === "https:" || (url.protocol === "http:" && local);
  if (!secure || url.origin.length > MAX_ORIGIN_LEN) return null;
  return url.origin;
}

/** The web context of a sender, or null when it is not a secure context. */
export function webContext(
  frameUrl: string | undefined,
  tabUrl: string | undefined,
): WebContext | null {
  const origin = originIfSecure(frameUrl);
  const topOrigin = originIfSecure(tabUrl);
  return origin === null || topOrigin === null ? null : { origin, topOrigin };
}
