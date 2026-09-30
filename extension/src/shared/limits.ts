/** Numbers and names both sides of the extension must agree on. */

/** The one protocol version this extension speaks. */
export const PROTOCOL_VERSION = 1;

/** Close the native port after this long without traffic or open requests. */
export const IDLE_CLOSE_MS = 60_000;

/** The app must answer `hello` within this time (protocol.md §8). */
export const HELLO_TIMEOUT_MS = 3_000;

/** `status` never waits for a person, so it gets a short leash. */
export const STATUS_TIMEOUT_MS = 10_000;

/** Longest page-chosen request id; leaves room for "<tab>.<frame>." in 64. */
export const MAX_PAGE_ID_LEN = 40;

/** Open requests per connection the app accepts (protocol.md §8). */
export const MAX_IN_FLIGHT = 16;

/** `source` of messages posted by the SDK. */
export const PAGE_SOURCE = "websign-page";

/** `source` of messages posted by the content script. */
export const EXTENSION_SOURCE = "websign-extension";

const PAGE_ID = /^[A-Za-z0-9._:-]+$/;

/** Whether `value` is usable as a page request id (and inside a native id). */
export function isPageId(value: unknown): value is string {
  return typeof value === "string" && value.length <= MAX_PAGE_ID_LEN && PAGE_ID.test(value);
}
