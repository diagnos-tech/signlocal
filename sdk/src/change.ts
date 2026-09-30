import type { Status } from "./types";

/**
 * Calls `listener` whenever the status may have changed (extension
 * announced itself, app became reachable or outdated). Returns an
 * unsubscribe function.
 */
export function onChange(listener: (status: Status) => void): () => void {
  void listener;
  throw new Error("unimplemented: SPEC.md §8");
}
