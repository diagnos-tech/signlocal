/**
 * Page ↔ background relay: accepts only same-window, same-origin messages
 * with source "websign-page", rebuilds them field by field, and posts
 * replies back with source "websign-extension" to the page's own origin.
 */

/** Starts relaying for this frame. */
export function startRelay(): void {
  throw new Error("unimplemented: SPEC.md §3.1");
}
