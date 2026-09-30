/** What `certificates()` and `sign()` share: reach the extension, ask, turn error replies into exceptions. */

import { type Announcement, discover, notify, PROTOCOL_VERSION, send } from "./channel.js";
import { abortedError, knownCode, malformedReply, WebSignError } from "./errors.js";
import type { PageReply, PageRequest } from "./generated/index.js";

/** `promise`, or `Aborted` as soon as `signal` fires. */
function untilAborted<T>(promise: Promise<T>, signal: AbortSignal | undefined): Promise<T> {
  if (signal === undefined) return promise;
  return new Promise((resolve, reject) => {
    const onAbort = () => reject(abortedError());
    signal.addEventListener("abort", onAbort, { once: true });
    promise.then(resolve, reject).finally(() => signal.removeEventListener("abort", onAbort));
  });
}

/**
 * The extension's announcement, once it is known to speak our protocol.
 *
 * @throws {WebSignError} `Aborted` when `signal` fires first (nothing is
 * posted if it already had); `ExtensionMissing` when nobody answers;
 * `ClientOutdated` when the extension only speaks newer protocols than this
 * SDK (the site must update `@websign/sdk`); `ExtensionOutdated` when it only
 * speaks older ones (the person must update the extension).
 */
export async function connect(signal?: AbortSignal): Promise<Announcement> {
  if (signal?.aborted) throw abortedError();
  const announcement = await untilAborted(discover(), signal);
  if (announcement === null) {
    throw new WebSignError(
      "ExtensionMissing",
      "The WebeSign extension did not answer: it is not installed, or disabled for this site.",
    );
  }
  const { min, max } = announcement.protocols;
  if (min > PROTOCOL_VERSION) {
    throw new WebSignError(
      "ClientOutdated",
      `The extension speaks protocol ${min}-${max}; this @websign/sdk speaks ${PROTOCOL_VERSION}.`,
      { installed: String(PROTOCOL_VERSION), required: String(min) },
    );
  }
  if (max < PROTOCOL_VERSION) {
    throw new WebSignError(
      "ExtensionOutdated",
      `The extension speaks protocol ${min}-${max}; this @websign/sdk needs ${PROTOCOL_VERSION}.`,
      { installed: String(max), required: String(PROTOCOL_VERSION) },
    );
  }
  return announcement;
}

type Final = Exclude<PageReply, { type: "sign.need_digest" }>;
type NeedDigestReply = Extract<PageReply, { type: "sign.need_digest" }>;

/**
 * Sends `request` and resolves with its final reply of type `expected`.
 * `onNeedDigest` sees the intermediate `sign.need_digest` replies.
 *
 * @throws {WebSignError} the app's or extension's error (unknown codes as
 * `Internal`), `Aborted` when `signal` fires, `Internal` for a reply of the
 * wrong kind.
 */
export async function ask<T extends Final["type"]>(
  request: PageRequest,
  expected: T,
  options: {
    readonly signal?: AbortSignal | undefined;
    readonly onNeedDigest?: (reply: NeedDigestReply, id: string) => void;
  } = {},
): Promise<Extract<Final, { type: T }>> {
  let final: Final | undefined;
  await send(
    request,
    (reply, id) => {
      if (reply.type === "sign.need_digest") options.onNeedDigest?.(reply, id);
      else final = reply;
    },
    options.signal,
  ).done;
  if (final?.type === "error") {
    const code = knownCode(final.code);
    if (code === "AppMissing" || code === "AppOutdated") notify();
    throw new WebSignError(code, final.message, final.details);
  }
  if (final?.type !== expected) throw malformedReply(`${final?.type} instead of ${expected}`);
  return final as Extract<Final, { type: T }>;
}

/**
 * Runs `convert` on a final reply; any malformation it trips over (a missing
 * field, bad Base64) becomes `Internal` instead of a `TypeError`.
 */
export function readReply<T>(convert: () => T): T {
  try {
    return convert();
  } catch {
    throw malformedReply("a reply the SDK cannot read");
  }
}
