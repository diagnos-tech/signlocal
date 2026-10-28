// The link to the current steps: this page (in the visitor's language) plus the choices, opening
// on "Your steps". Meant to be opened on another computer.

import type { Choices } from "../platform/types.ts";
import { serializeUrl } from "./url.ts";

export function sharedUrl(
  choices: Choices,
  page: Pick<Location, "origin" | "pathname"> = location,
): string {
  return `${page.origin}${page.pathname}?${serializeUrl(choices, "result")}`;
}

/** Copies text to the clipboard; false when the browser refuses. */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
