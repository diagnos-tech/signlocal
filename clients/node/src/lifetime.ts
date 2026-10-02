/** How the `websign connect` child relates to the caller's own process lifetime. */

import type { ChildProcess } from "node:child_process";

interface Referable {
  ref?: () => void;
  unref?: () => void;
}

/**
 * Lets Node exit while no answer is owed: an idle connection whose caller
 * forgot `close()` must not keep the program alive until the app's 300 s idle
 * timeout. While a request is in flight (the person may still be typing a
 * PIN) or `close()` waits for a graceful exit, the child and its pipes hold
 * the event loop as usual.
 */
export function holdEventLoop(child: ChildProcess, hold: boolean): void {
  for (const handle of [child, child.stdin, child.stdout] as Array<Referable | null>) {
    if (hold) handle?.ref?.();
    else handle?.unref?.();
  }
}

/**
 * Registers a last-chance kill for when the caller's process exits with the
 * child still running; returns the unregister function. `SIGKILL` because
 * nothing asynchronous can run in an `exit` handler; on Windows every signal
 * maps to `TerminateProcess`, which is the same outcome.
 */
export function killOnExit(child: ChildProcess): () => void {
  const kill = () => child.kill("SIGKILL");
  process.on("exit", kill);
  return () => process.off("exit", kill);
}
