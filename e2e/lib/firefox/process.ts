/**
 * Starting and ending the Firefox process. On Windows `firefox.exe` is a
 * launcher that starts the browser as a child process and exits at once;
 * `--wait-for-browser` keeps it alive until the browser exits, so the
 * process we hold stands for the browser (its exit code means the browser
 * ended), and a kill of its tree reaches the browser that holds the profile.
 */

import { type ChildProcess, execFile } from "node:child_process";
import { platform } from "node:os";

/** Arguments every launch needs on this OS. */
export const PLATFORM_ARGS: readonly string[] =
  platform() === "win32" ? ["--wait-for-browser"] : [];

/** How long each step of `exit` waits before the next, harder one. */
const GRACE = 10_000;

/**
 * Ends Firefox: waits for it when it was `asked` to close (BiDi
 * `browser.close`), then terminates it, then kills it. Resolves once the
 * process is gone, so the profile folder is free to remove.
 */
export async function exit(firefox: ChildProcess, asked: boolean): Promise<void> {
  if (ended(firefox)) return;
  const exited = new Promise<void>((done) => firefox.once("exit", () => done()));
  if (asked && (await within(exited, GRACE))) return;
  terminate(firefox, false);
  if (await within(exited, GRACE)) return;
  terminate(firefox, true);
  await exited;
}

function ended(child: ChildProcess): boolean {
  return child.exitCode !== null || child.signalCode !== null;
}

/** Whether `promise` settles within `ms`. */
async function within(promise: Promise<void>, ms: number): Promise<boolean> {
  let timer: NodeJS.Timeout | undefined;
  const late = new Promise<false>((done) => {
    timer = setTimeout(() => done(false), ms);
  });
  const settled = await Promise.race([promise.then(() => true as const), late]);
  clearTimeout(timer);
  return settled;
}

/**
 * Signals the process, or on Windows ends its whole tree: signals there are
 * a plain TerminateProcess of the launcher, which would orphan the browser.
 */
function terminate(child: ChildProcess, force: boolean): void {
  if (platform() !== "win32" || child.pid === undefined) {
    child.kill(force ? "SIGKILL" : "SIGTERM");
    return;
  }
  const args = ["/pid", String(child.pid), "/T", ...(force ? ["/F"] : [])];
  execFile("taskkill", args, () => undefined);
}
