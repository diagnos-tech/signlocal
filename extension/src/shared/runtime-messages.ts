/**
 * Messages that travel through `runtime.sendMessage` / `tabs.sendMessage`
 * between the content script, the background and the popup. Every kind is
 * checked against its sender on receipt: the content script shares a process
 * with the page and the popup is the only other legitimate caller.
 */

import type { PageReply } from "../generated";

/** Content script → background: a rebuilt page request. */
export interface RelayRequest {
  readonly kind: "websign-request";
  /** Random per content-script instance (one per document); see {@link RelayGone}. */
  readonly document: string;
  readonly id: string;
  readonly message: unknown;
}

/** Content script → background: this document is going away; cancel what it opened. */
export interface RelayGone {
  readonly kind: "websign-gone";
  readonly document: string;
}

/** Background → content script: a reply for the page. */
export interface RelayReply {
  readonly kind: "websign-reply";
  /** The requesting document's token; other documents in the frame drop the reply. */
  readonly document: string;
  readonly id: string;
  readonly message: PageReply;
}

/** Background → top frame's content script: "what is your origin?" (§2.3). */
export interface OriginQuery {
  readonly kind: "websign-origin";
}

/** Popup → background. */
export interface PopupRequest {
  readonly kind: "websign-popup";
  readonly op: "probe" | "diagnostics";
}

/** Background → popup, for `probe`. `details` carries the versions of an `AppOutdated`. */
export type ProbeResult =
  | { readonly ok: true; readonly appVersion: string }
  | {
      readonly ok: false;
      readonly code: string;
      readonly details?: { readonly installed?: string; readonly required?: string };
    };
