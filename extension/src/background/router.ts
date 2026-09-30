/**
 * Routes page requests to the app over the shared native port and replies
 * back to the right frame. Page ids are only unique per page, so each
 * (tab, frame, page id) gets a connection-wide native id (requests.ts);
 * every app message is delivered only to the entry with exactly that id.
 */

import { browser } from "wxt/browser";
import type {
  AppEnvelope,
  AppInfo,
  ClientEnvelope,
  ExtensionInfo,
  PageReply,
  PageRequest,
  WebContext,
} from "../generated";
import { MIN_APP_VERSION } from "../generated";
import { isPageId, MAX_IN_FLIGHT, PROTOCOL_VERSION, STATUS_TIMEOUT_MS } from "../shared/limits";
import { isOlder } from "../shared/version";
import { closedCode } from "./app-error";
import { detectBrowser } from "./browser";
import { type Connection, connect } from "./connection";
import { continuation, type Entry, entries, failure, finish, nativeId, toError } from "./requests";

/** Where a page request came from, as the browser reported it. */
export interface PageSender {
  readonly tabId: number;
  readonly frameId: number;
  readonly context: WebContext;
  /** The content script's per-document token, when it sent one. */
  readonly document?: string;
}

const attached = new WeakSet<Connection>();

async function extensionInfo(): Promise<ExtensionInfo> {
  return { version: browser.runtime.getManifest().version, browser: (await detectBrowser()).name };
}

/** What the page sees for `status`: it never fails, missing pieces are just absent. */
async function statusReply(app: AppInfo | undefined, outdated: boolean, remembered: boolean) {
  const reply: PageReply = {
    type: "status",
    extension: await extensionInfo(),
    appOutdated: outdated,
    remembered,
  };
  return app === undefined ? reply : { ...reply, app };
}

function onAppMessage(conn: Connection, message: AppEnvelope): void {
  const entry = entries.get(message.id);
  if (entry === undefined || entry.conn !== conn) return;
  const { v: _v, id: _id, ...body } = message;
  switch (message.type) {
    case "sign.need_digest":
      entry.reply(body as PageReply);
      return;
    case "status":
      finish(message.id, entry);
      void statusReply(message.app, false, message.remembered).then(entry.reply);
      return;
    case "choose.result":
    case "sign.result":
    case "error":
      finish(message.id, entry);
      entry.reply(body as PageReply);
      return;
    default:
      finish(message.id, entry);
      entry.reply(failure("Internal", "the app sent an unexpected reply"));
  }
}

function onClosed(conn: Connection, heard: boolean): void {
  for (const [id, entry] of [...entries]) {
    if (entry.conn !== conn) continue;
    finish(id, entry);
    if (entry.app !== undefined) {
      void statusReply(entry.app, false, false).then(entry.reply);
    } else {
      entry.reply(failure(closedCode(heard), "the app closed the connection"));
    }
  }
}

function attach(conn: Connection): void {
  if (attached.has(conn)) return;
  attached.add(conn);
  conn.onMessage((message) => onAppMessage(conn, message));
  void conn.closed.then(({ heard }) => onClosed(conn, heard));
}

/**
 * Connects and sends. An app older than MIN_APP_VERSION is never sent page
 * requests (not even `status`): it may mishandle them, which is the reason
 * for the minimum.
 */
async function start(
  sender: PageSender,
  id: string,
  request: PageRequest,
  entry: Entry,
): Promise<void> {
  const isStatus = request.type === "status";
  let conn: Connection;
  try {
    conn = await connect("page");
  } catch (error) {
    finish(id, entry);
    entry.reply(isStatus ? await statusReply(undefined, false, false) : toError(error));
    return;
  }
  if (entries.get(id) !== entry) return;
  const { app } = conn.hello;
  if (isOlder(app.version, MIN_APP_VERSION)) {
    finish(id, entry);
    if (isStatus) return entry.reply(await statusReply(app, true, false));
    return entry.reply({
      type: "error",
      code: "AppOutdated",
      message: "the app is older than this extension supports",
      details: { installed: app.version, required: MIN_APP_VERSION },
    });
  }
  attach(conn);
  entry.conn = conn;
  if (isStatus) {
    entry.app = app;
    entry.timer = setTimeout(() => {
      finish(id, entry);
      void statusReply(app, false, false).then(entry.reply);
    }, STATUS_TIMEOUT_MS);
  }
  try {
    conn.send({ ...request, v: PROTOCOL_VERSION, id, web: sender.context } as ClientEnvelope);
  } catch (error) {
    finish(id, entry);
    entry.reply(toError(error));
  }
}

/** Forwards `request`; `reply` receives every message for it until the final one. */
export function route(
  sender: PageSender,
  pageId: string,
  request: PageRequest,
  reply: (message: PageReply) => void,
): void {
  if (!isPageId(pageId)) {
    reply(failure("InvalidRequest", "the request id is not valid"));
    return;
  }
  const id = nativeId(sender.tabId, sender.frameId, pageId);
  if (request.type === "cancel" || request.type === "sign.digest") {
    continuation(id, request);
    return;
  }
  if (entries.has(id)) {
    reply(failure("InvalidRequest", "a request with this id is already open"));
    return;
  }
  if (entries.size >= MAX_IN_FLIGHT) {
    reply(failure("Busy", "too many open requests"));
    return;
  }
  const entry: Entry = {
    tabId: sender.tabId,
    frameId: sender.frameId,
    topOrigin: sender.context.topOrigin,
    document: sender.document,
    reply,
    conn: null,
    ...(request.type === "sign.begin" ? { hash: request.hash } : {}),
  };
  entries.set(id, entry);
  void start(sender, id, request, entry).catch((error) => {
    finish(id, entry);
    reply(toError(error));
  });
}
