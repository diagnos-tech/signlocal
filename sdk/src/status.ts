import { discover, send } from "./channel.js";
import type { PageReply } from "./generated/index.js";
import { isCompatible } from "./request.js";
import type { Status } from "./types.js";

/**
 * The extension answers `status` itself; this only bounds a hung extension.
 * It outlasts the extension's worst case, the 8 s it gives an app launched
 * for the first time plus the 1.5 s it waits for the app's answer, so a slow
 * but healthy first start is not reported as a missing app.
 */
const STATUS_TIMEOUT_MS = 10_000;

type AppState = Status["app"];

function build(extension: Status["extension"], app: AppState, remembered = false): Status {
  return {
    extension,
    app,
    remembered,
    ready: extension.installed && app.installed && !app.outdated,
  };
}

const NO_APP: AppState = { installed: false, outdated: false };

function fromReply(extension: Status["extension"], reply: PageReply | undefined): Status {
  if (reply?.type === "status") {
    const { app, appOutdated, remembered } = reply;
    const state = app ? { installed: true, version: app.version } : { installed: false };
    return build(extension, { ...state, outdated: appOutdated }, remembered);
  }
  if (reply?.type === "error") {
    return build(extension, {
      installed: reply.code !== "AppMissing",
      outdated: reply.code === "AppOutdated",
    });
  }
  return build(extension, NO_APP);
}

/**
 * The state of the extension and the app, without opening any window. Never
 * rejects: whatever is missing shows as `installed: false`.
 */
export async function status(): Promise<Status> {
  try {
    const announcement = await discover();
    if (announcement === null) return build({ installed: false }, NO_APP);
    const extension = { installed: true, version: announcement.extension.version };
    if (!isCompatible(announcement)) return build(extension, NO_APP);

    const timeout = new AbortController();
    const timer = setTimeout(() => timeout.abort(), STATUS_TIMEOUT_MS);
    const replies: PageReply[] = [];
    try {
      await send({ type: "status" }, (reply) => replies.push(reply), timeout.signal).done;
    } catch {
      // Timed out: the extension is there but the app did not answer.
    } finally {
      clearTimeout(timer);
    }
    return fromReply(extension, replies.at(-1));
  } catch {
    return build({ installed: false }, NO_APP);
  }
}
