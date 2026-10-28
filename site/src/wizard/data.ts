// What the wizard needs to know that is not text: which devices exist, where each browser gets
// the extension, and the oldest app the extension accepts. Rendered into the page at build time
// as a JSON block (components/wizard/Wizard.astro).

import type { BrowserId, Channel } from "../platform/types.ts";

export interface WizardData {
  /** devices.json ids a person can pick, with their kind. */
  readonly devices: Readonly<Record<string, "token" | "card">>;
  readonly channels: Readonly<Record<BrowserId, Channel>>;
  readonly minAppVersion: string;
}

export function readWizardData(): WizardData {
  const block = document.getElementById("wizard-data");
  if (!block?.textContent) throw new Error("The page has no wizard-data block");
  return JSON.parse(block.textContent) as WizardData;
}
