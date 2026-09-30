import type { Status } from "./types";

/** The state of the extension and the app, without opening any window. Never rejects. */
export function status(): Promise<Status> {
  throw new Error("unimplemented: SPEC.md §4");
}
