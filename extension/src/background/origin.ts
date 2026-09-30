/**
 * The requesting origins, from the browser's MessageSender (never from the
 * payload). Refuses non-secure contexts with InsecureOrigin before anything
 * reaches the app: http other than localhost, file:, data:, extensions.
 */

import type { WebContext } from "../generated";

/** The protocol's origin limit (protocol.md §8); longer origins are refused. */
const MAX_ORIGIN_LEN = 512;

function isLoopbackV4(host: string): boolean {
  const octets = host.split(".");
  return (
    octets.length === 4 &&
    octets[0] === "127" &&
    octets.every((part) => /^\d{1,3}$/.test(part) && Number(part) <= 255)
  );
}

/** https anywhere; http only for names and addresses that never leave the machine. */
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
    host === "localhost" || host.endsWith(".localhost") || host === "[::1]" || isLoopbackV4(host);
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
