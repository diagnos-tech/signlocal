/**
 * Answers the toolbar popup: "is the app there?" and "open diagnostics".
 * Only extension pages (no tab) may ask; a content script is a page's proxy.
 */

import type { Browser } from "wxt/browser";
import { browser } from "wxt/browser";
import { MIN_APP_VERSION } from "../generated";
import type { PopupRequest, ProbeResult } from "../shared/runtime-messages";
import { isOlder } from "../shared/version";
import { AppError } from "./app-error";
import { showHealth } from "./badge";
import { connect } from "./connection";
import { ask } from "./rpc";

const DIAGNOSTICS_TIMEOUT_MS = 5_000;

async function probe(): Promise<ProbeResult> {
  try {
    const conn = await connect("popup");
    const appVersion = conn.hello.app.version;
    showHealth(!isOlder(appVersion, MIN_APP_VERSION));
    return { ok: true, appVersion };
  } catch (error) {
    showHealth(false);
    return failed(error);
  }
}

/** The probe result of a failure; versions travel along for `AppOutdated`. */
function failed(error: unknown): ProbeResult {
  if (!(error instanceof AppError)) return { ok: false, code: "Internal" };
  const details = error.details;
  const versions: { installed?: string; required?: string } = {};
  if (typeof details?.installed === "string") versions.installed = details.installed;
  if (typeof details?.required === "string") versions.required = details.required;
  return Object.keys(versions).length > 0
    ? { ok: false, code: error.code, details: versions }
    : { ok: false, code: error.code };
}

async function openDiagnostics(): Promise<ProbeResult> {
  try {
    const reply = await ask(
      await connect("popup"),
      { type: "diagnostics.open" },
      DIAGNOSTICS_TIMEOUT_MS,
    );
    if (reply.type === "error") return { ok: false, code: reply.code };
    return { ok: true, appVersion: "" };
  } catch (error) {
    return failed(error);
  }
}

function fromPopup(sender: Browser.runtime.MessageSender): boolean {
  return (
    sender.id === browser.runtime.id &&
    sender.tab === undefined &&
    sender.url?.startsWith(browser.runtime.getURL("/")) === true
  );
}

/** Returns true when it will answer through `respond` (the listener must then return true). */
export function handlePopupMessage(
  message: unknown,
  sender: Browser.runtime.MessageSender,
  respond: (result: ProbeResult) => void,
): boolean {
  const request = message as Partial<PopupRequest> | null;
  if (request?.kind !== "websign-popup" || !fromPopup(sender)) return false;
  const work = request.op === "diagnostics" ? openDiagnostics() : probe();
  void work.then(respond);
  return true;
}
