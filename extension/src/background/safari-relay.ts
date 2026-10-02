/**
 * The Safari app extension's relay contract (`safari/SPEC.md` §1–2): the
 * requests the background sends through `runtime.sendNativeMessage`, and a
 * strict reader for the replies. A reply of any other shape is a bug on one
 * side of the contract, never something to guess at.
 */

import type { AppEnvelope, ErrorCode } from "../generated";

export const RELAY_VERSION = 1;
/** How long the appex may hold a poll open (it accepts 0–10 000 ms). */
export const POLL_WAIT_MS = 5_000;
/**
 * Extra time Safari and the appex get to deliver any reply. A reply that
 * never comes means the appex is stuck or gone; waiting forever would hang
 * every request on the connection.
 */
export const REPLY_GRACE_MS = 5_000;
/** Longest wait for the reply to `open` or `send` (they are never parked). */
export const CALL_TIMEOUT_MS = 10_000;
/** The appex keeps at most 64 undelivered host messages per session. */
const MAX_MESSAGES = 64;

export type RelayRequest =
  | { relay: 1; op: "open" }
  | { relay: 1; op: "send"; session: string; message: object }
  | { relay: 1; op: "poll"; session: string; wait: number }
  | { relay: 1; op: "close"; session: string };

type RelayErrorCode = "BadRequest" | "NoSession" | "TooManySessions" | "HostMissing";

export type RelayReply =
  | { kind: "session"; session: string; messages: AppEnvelope[]; open: boolean }
  | { kind: "error"; code: RelayErrorCode };

/** The canonical upper-case UUID the appex names sessions with. */
const SESSION = /^[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}$/;
const ERROR_CODES: ReadonlySet<string> = new Set<RelayErrorCode>([
  "BadRequest",
  "NoSession",
  "TooManySessions",
  "HostMissing",
]);

function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

function hasExactly(value: Record<string, unknown>, keys: readonly string[]): boolean {
  const own = Object.keys(value);
  return own.length === keys.length && keys.every((key) => Object.hasOwn(value, key));
}

function isEnvelope(value: unknown): value is AppEnvelope {
  return isPlainObject(value) && typeof value.id === "string" && typeof value.type === "string";
}

/** The reply as the contract defines it, or null for anything else. */
export function parseRelayReply(value: unknown): RelayReply | null {
  if (!isPlainObject(value) || value.relay !== RELAY_VERSION) return null;
  if (hasExactly(value, ["relay", "error"])) {
    const { error } = value;
    return typeof error === "string" && ERROR_CODES.has(error)
      ? { kind: "error", code: error as RelayErrorCode }
      : null;
  }
  if (!hasExactly(value, ["relay", "session", "messages", "open"])) return null;
  const { session, messages, open } = value;
  if (typeof session !== "string" || !SESSION.test(session) || typeof open !== "boolean") {
    return null;
  }
  if (!Array.isArray(messages) || messages.length > MAX_MESSAGES) return null;
  if (!messages.every(isEnvelope)) return null;
  return { kind: "session", session, messages, open };
}

/**
 * The protocol code a relay error stands for, as `safari/SPEC.md` §2
 * assigns them; `null` for `NoSession`, which is a closed port and is
 * classified like one (by whether the app spoke).
 */
export function relayErrorCode(code: RelayErrorCode): ErrorCode | null {
  switch (code) {
    case "HostMissing":
      return "AppMissing";
    case "NoSession":
      return null;
    default:
      return "Internal";
  }
}
