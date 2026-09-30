/** Which browser hosts the extension (for `hello` and install links). */

import type { BrowserName } from "../generated";

/** Detected from `runtime.getBrowserInfo` (Firefox), UA-CH brands, or the UA string. */
export function detectBrowser(): Promise<{ readonly name: BrowserName; readonly version: string }> {
  throw new Error("unimplemented: SPEC.md §2.5");
}
