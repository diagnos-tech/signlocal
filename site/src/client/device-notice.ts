// The phone notice: Share (only where the browser can share), Copy link, and the e-mail link
// kept in step with the URL being shared.

import type { MessageKey } from "../generated/message-keys.ts";
import { readStrings } from "./strings.ts";

export const STRINGS = [
  "device.email_subject",
  "device.email_body",
  "common.copied",
  "common.copy_failed",
] as const satisfies readonly MessageKey[];

export function mailtoFor(url: string, strings: { subject: string; body: string }): string {
  const body = strings.body.replace("{url}", url);
  return `mailto:?subject=${encodeURIComponent(strings.subject)}&body=${encodeURIComponent(body)}`;
}

/** Wires the notice inside `element`; `currentUrl` says what to share right now. */
export function initDeviceNotice(element: HTMLElement, currentUrl: () => string): void {
  const strings = readStrings("device", STRINGS);
  const share = element.querySelector<HTMLButtonElement>("[data-share]");
  const copy = element.querySelector<HTMLButtonElement>("[data-copy-link]");
  const label = element.querySelector<HTMLElement>("[data-copy-link-label]");
  const email = element.querySelector<HTMLAnchorElement>("[data-email-link]");

  const refreshEmail = () => {
    if (email) {
      email.href = mailtoFor(currentUrl(), {
        subject: strings["device.email_subject"],
        body: strings["device.email_body"],
      });
    }
  };
  refreshEmail();

  if (share && typeof navigator.share === "function") {
    share.hidden = false;
    share.addEventListener("click", () => {
      navigator.share({ title: document.title, url: currentUrl() }).catch(() => undefined);
    });
  }

  copy?.addEventListener("click", async () => {
    refreshEmail();
    const original = label?.textContent ?? "";
    try {
      await navigator.clipboard.writeText(currentUrl());
      if (label) label.textContent = strings["common.copied"];
    } catch {
      if (label) label.textContent = strings["common.copy_failed"];
    }
    setTimeout(() => {
      if (label) label.textContent = original;
    }, 2000);
  });
  element.addEventListener("focusin", refreshEmail);
}
