// Draws the test page: which setup state shows, the install links for this browser, and the
// result of a signature or the error. Every element is already in the page; this fills and
// shows them.

import type { Certificate, VerificationCode } from "@signlocal/sdk";
import { type ErrorLike, errorText } from "@signlocal/sdk/messages";
import type { Check } from "./crypto.ts";
import { keyText, type SetupState, type SetupStatus, setupState } from "./state.ts";

const $ = (id: string): HTMLElement => {
  const element = document.getElementById(id);
  if (!element) throw new Error(`The test page has no #${id}`);
  return element;
};

export function renderSetup(status: SetupStatus | null, busy: boolean, canSign: boolean): void {
  const state: SetupState = setupState(status);
  for (const element of document.querySelectorAll<HTMLElement>("#setup [data-state]")) {
    element.hidden = element.dataset.state !== state;
  }
  ($("sign") as HTMLButtonElement).disabled = state !== "ready" || !canSign || busy;
  $("v-ext").textContent = status?.extension.version ?? "-";
  $("v-app").textContent = status?.app.version ?? "-";
}

/** Points the install buttons at the wizard with this computer's system and browser. */
export function renderInstallLinks(href: string): void {
  for (const id of ["install-ext", "install-app", "update-app"])
    ($(id) as HTMLAnchorElement).href = href;
}

function drawIdenticon(code: VerificationCode): void {
  const icon = $("r-icon");
  icon.className = `identicon id-${code.colorIndex}`;
  for (const [index, cell] of [...icon.children].entries()) {
    cell.classList.toggle("on", code.cells[index] === true);
  }
}

export interface Success {
  readonly certificate: Certificate;
  readonly algorithm: string;
  readonly hash: string;
  readonly code: VerificationCode;
  readonly check: Check;
}

export function renderSuccess(done: Success): void {
  $("result").hidden = false;
  $("result-ok").hidden = false;
  $("result-error").hidden = true;
  $("r-name").textContent = done.certificate.displayName;
  $("r-alg").textContent = `${done.algorithm} · ${done.hash}`;
  $("r-key").textContent = keyText(done.certificate.key);
  $("r-code").textContent = done.code.text;
  drawIdenticon(done.code);
  $("ok-title").hidden = done.check === "invalid";
  $("bad-title").hidden = done.check !== "invalid";
  $("result-ok").classList.toggle("callout--warning", done.check === "invalid");
  $("result-ok").classList.toggle("callout--success", done.check !== "invalid");
  for (const name of ["verified", "invalid", "unsupported"] as const)
    $(`v-${name}`).hidden = done.check !== name;
}

export function renderFailure(
  code: string,
  error: ErrorLike | undefined,
  language: string,
  generic: string,
): void {
  $("result").hidden = false;
  $("result-ok").hidden = true;
  $("result-error").hidden = false;
  const texts = error ? errorText(error, language) : undefined;
  $("e-title").textContent = texts?.title ?? generic;
  $("e-body").textContent = texts?.body ?? "";
  $("e-code").textContent = code;
}

export function renderSummary(summary: string): void {
  $("summary").textContent = summary;
}
