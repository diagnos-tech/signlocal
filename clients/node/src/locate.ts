import { currentEnvironment, search } from "./candidates.js";

/**
 * The `websign` executable: `WEBSIGN_EXECUTABLE` if it exists, else `websign`
 * on `PATH`, else the per-OS install locations of
 * docs/architecture/packaging-and-release.md §Install locations, in that
 * order. `undefined` when none exists.
 *
 * The override comes first so tests and side-by-side installs can pin a build;
 * `PATH` beats fixed locations because the person's shell already chose it.
 */
export function findExecutable(): string | undefined {
  return search(currentEnvironment());
}
