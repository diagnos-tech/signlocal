/**
 * The requesting origins, from the browser's MessageSender (never from the
 * payload). Refuses non-secure contexts with InsecureOrigin before anything
 * reaches the app: http other than localhost, file:, data:, extensions.
 */

import type { WebContext } from "../generated";

/** The web context of a sender, or null when it is not a secure context. */
export function webContext(
  frameUrl: string | undefined,
  tabUrl: string | undefined,
): WebContext | null {
  void frameUrl;
  void tabUrl;
  throw new Error("unimplemented: SPEC.md §2.3");
}
