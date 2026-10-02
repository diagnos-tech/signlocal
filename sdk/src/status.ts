import { discover, PROTOCOL_VERSION, send } from "./channel.js";
import type { PageReply } from "./generated/index.js";
import type { Status, StatusProblem } from "./types.js";

/**
 * The extension answers `status` itself; this only bounds a hung extension.
 * It outlasts the extension's worst case, the 8 s it gives an app launched
 * for the first time plus the 1.5 s it waits for the app's answer, so a slow
 * but healthy first start is not reported as a missing app.
 */
const STATUS_TIMEOUT_MS = 10_000;

type Extension = Status["extension"];
type AppState = Status["app"];

const NO_EXTENSION: Extension = { installed: false };
const NO_APP: AppState = { installed: false, outdated: false };

/** The status, with `ready` and `problem` derived so they can never disagree. */
function build(
  extension: Extension,
  app: AppState,
  remembered = false,
  // Only an incompatible protocol range passes this; the rest follows from the facts.
  protocol?: StatusProblem,
): Status {
  const problem: StatusProblem | undefined = !extension.installed
    ? "ExtensionMissing"
    : (protocol ?? (!app.installed ? "AppMissing" : app.outdated ? "AppOutdated" : undefined));
  return { extension, app, remembered, ready: !problem, ...(problem && { problem }) };
}

function fromReply(extension: Extension, reply: PageReply | undefined): Status {
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
 * rejects: whatever is missing shows as `installed: false`, and `problem`
 * names the first thing to fix. Call it on load to show or hide your Sign
 * button; {@link onChange} tells you when it changes.
 *
 * @example
 * const { ready, problem } = await status();
 * button.hidden = !ready;
 * if (problem === "ExtensionMissing") installLink.href = installUrl();
 */
export async function status(): Promise<Status> {
  try {
    const announcement = await discover();
    if (announcement === null) return build(NO_EXTENSION, NO_APP);
    const extension = { installed: true, version: announcement.extension.version };
    const { min, max } = announcement.protocols;
    if (min > PROTOCOL_VERSION) return build(extension, NO_APP, false, "ClientOutdated");
    if (max < PROTOCOL_VERSION) return build(extension, NO_APP, false, "ExtensionOutdated");

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
    return build(NO_EXTENSION, NO_APP);
  }
}
