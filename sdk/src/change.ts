import { subscribe } from "./channel.js";
import { status } from "./status.js";
import type { Status } from "./types.js";

/** Bursts (an announcement in every frame, a failing request retried) collapse into one call. */
const DEBOUNCE_MS = 250;

/**
 * Calls `listener` whenever the status may have changed: the extension
 * announced itself (installed or updated while the page is open), or a
 * request failed with `AppMissing`/`AppOutdated` (the app was removed or
 * needs an update). The listener receives a fresh {@link status}; it is not
 * called for the state at subscription time, so call `status()` once
 * yourself. Returns an unsubscribe function (safe to call twice).
 *
 * @example
 * const stop = onChange((s) => setReady(s.ready));
 */
export function onChange(listener: (status: Status) => void): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let active = true;
  const unsubscribe = subscribe(() => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const current = await status();
      if (active) listener(current);
    }, DEBOUNCE_MS);
  });
  return () => {
    active = false;
    clearTimeout(timer);
    unsubscribe();
  };
}
