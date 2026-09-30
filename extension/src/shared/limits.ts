/**
 * Numbers and names both sides of the extension must agree on. The ones the
 * app shares come from websign-protocol through limits.gen.ts; the rest are
 * the extension's own policy.
 */

import {
  APP_RESPONSE_TIMEOUT_MS,
  EXTENSION_IDLE_CLOSE_MS,
  EXTENSION_SOURCE,
  MAX_IN_FLIGHT_PER_CONNECTION,
  PAGE_SOURCE,
  PROTOCOL_VERSION,
} from "./limits.gen";

export { EXTENSION_SOURCE, PAGE_SOURCE, PROTOCOL_VERSION };

/** Close the native port after this long without traffic or open requests. */
export const IDLE_CLOSE_MS = EXTENSION_IDLE_CLOSE_MS;

/** The app must answer `hello` within this time once it has answered before. */
export const HELLO_TIMEOUT_MS = APP_RESPONSE_TIMEOUT_MS;

/**
 * `hello` budget until the app has answered once in this background's life.
 * The first launch after an install or update can be slow for reasons that
 * are not faults: an unsigned binary scanned by SmartScreen, Gatekeeper or an
 * antivirus, a cold disk, the PKCS#11 driver loading. Giving up early there
 * would call a healthy install broken.
 */
export const FIRST_HELLO_TIMEOUT_MS = 8_000;

/**
 * `status` never waits for a person, so it gets a short leash once the app
 * is connected. The SDK's `status()` waits for the worst case, a first hello
 * plus this (sdk/src/status.ts), so the page always gets our answer.
 */
export const STATUS_TIMEOUT_MS = 1_500;

/** Longest page-chosen request id; leaves room for "<tab>.<frame>." in the app's limit. */
export const MAX_PAGE_ID_LEN = 40;

/** Open requests per connection the app accepts (protocol.md §8). */
export const MAX_IN_FLIGHT = MAX_IN_FLIGHT_PER_CONNECTION;

const PAGE_ID = /^[A-Za-z0-9._:-]+$/;

/** Whether `value` is usable as a page request id (and inside a native id). */
export function isPageId(value: unknown): value is string {
  return typeof value === "string" && value.length <= MAX_PAGE_ID_LEN && PAGE_ID.test(value);
}
