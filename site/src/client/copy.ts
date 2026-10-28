// Copy buttons: [data-copy-from="<id>"] copies the text of that element. The icon turns into a
// check for two seconds. If the Clipboard API refuses, the text is selected and a note says to
// copy it by hand. Nothing happens, and no button shows, without JavaScript.

import type { MessageKey } from "../generated/message-keys.ts";
import { readStrings } from "./strings.ts";

export const STRINGS = [
  "common.copy",
  "common.copied",
  "common.copy_failed",
] as const satisfies readonly MessageKey[];

const RESET_MS = 2000;

function selectText(element: HTMLElement): void {
  const selection = getSelection();
  if (!selection) return;
  const range = document.createRange();
  range.selectNodeContents(element);
  selection.removeAllRanges();
  selection.addRange(range);
}

/** The status line that announces the outcome; one per page, read politely. */
function announcer(): HTMLElement {
  let note = document.getElementById("copy-status");
  if (!note) {
    note = document.createElement("p");
    note.id = "copy-status";
    note.className = "sr-only";
    note.setAttribute("role", "status");
    document.body.append(note);
  }
  return note;
}

export function initCopyButtons(root: ParentNode = document): void {
  const strings = readStrings("copy", STRINGS);
  for (const button of root.querySelectorAll<HTMLButtonElement>("[data-copy-from]")) {
    const label = button.querySelector<HTMLElement>("[data-copy-label]");
    button.addEventListener("click", async () => {
      const source = document.getElementById(button.dataset.copyFrom ?? "");
      if (!source) return;
      let copied = true;
      try {
        await navigator.clipboard.writeText(source.textContent ?? "");
      } catch {
        copied = false;
        selectText(source);
      }
      announcer().textContent = copied ? strings["common.copied"] : strings["common.copy_failed"];
      if (!copied) return;
      button.dataset.state = "done";
      if (label) label.textContent = strings["common.copied"];
      setTimeout(() => {
        delete button.dataset.state;
        if (label) label.textContent = strings["common.copy"];
        announcer().textContent = "";
      }, RESET_MS);
    });
  }
}
