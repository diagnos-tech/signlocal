/**
 * Validation of what arrives on `window`. The page shares its window with
 * every script on it, so a message counts only when it has exactly the
 * shape the content script sends; anything else is dropped without a trace.
 */

import type { Announcement } from "./channel.js";
import type { ExtensionInfo, PageReply } from "./generated/index.js";

/** The `source` marker of the content script's messages. */
export const EXTENSION_SOURCE = "websign-extension";

/** A message from the content script that passed validation. */
export type Incoming =
  | ({ readonly kind: "announce" } & Announcement)
  | { readonly kind: "message"; readonly id: string; readonly reply: PageReply };

const REPLY_TYPES = ["status", "choose.result", "sign.need_digest", "sign.result", "error"];

/** Replies after which the request is over. */
export const FINAL_REPLY_TYPES = ["status", "choose.result", "sign.result", "error"];

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;

/** A protocol version or a digest sequence number: an integer from 1. */
const isPositiveInteger = (value: unknown): value is number =>
  Number.isInteger(value) && Number(value) >= 1;

/**
 * The fields the SDK acts on before converting a reply. `need_digest` is
 * checked here because it triggers site code (`prepare`); final replies are
 * converted under a guard that turns any malformation into `Internal`.
 */
function hasRequiredFields(reply: Record<string, unknown>): boolean {
  if (reply.type === "error") {
    return typeof reply.code === "string" && typeof reply.message === "string";
  }
  if (reply.type === "sign.need_digest") {
    return (
      isPositiveInteger(reply.seq) &&
      isObject(reply.certificate) &&
      typeof reply.hash === "string" &&
      typeof reply.algorithm === "string"
    );
  }
  return true;
}

/** The message if `data` is a well-formed message of the extension, else null. */
export function parseIncoming(data: unknown): Incoming | null {
  if (!isObject(data) || data.source !== EXTENSION_SOURCE) return null;
  if (data.kind === "announce") {
    const { extension, protocols } = data;
    if (!isObject(extension) || typeof extension.version !== "string") return null;
    if (typeof extension.browser !== "string") return null;
    if (
      !isObject(protocols) ||
      !isPositiveInteger(protocols.min) ||
      !isPositiveInteger(protocols.max)
    )
      return null;
    if (protocols.min > protocols.max) return null;
    return {
      kind: "announce",
      extension: {
        version: extension.version,
        browser: extension.browser as ExtensionInfo["browser"],
      },
      protocols: { min: protocols.min, max: protocols.max },
    };
  }
  if (data.kind === "message" && typeof data.id === "string") {
    const reply = data.message;
    if (!isObject(reply) || typeof reply.type !== "string") return null;
    if (!REPLY_TYPES.includes(reply.type) || !hasRequiredFields(reply)) return null;
    return { kind: "message", id: data.id, reply: reply as PageReply };
  }
  return null;
}
