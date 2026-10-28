// Reads what the browser says about itself, in the shape detect.ts takes. The only browser
// API touched is `navigator`; nothing is stored or sent.

import type { DetectionInput, HighEntropyValues } from "./detect.ts";
import { archFromHints } from "./detect.ts";
import type { Arch, BrowserId } from "./types.ts";

interface ClientHints {
  readonly platform?: string;
  readonly mobile?: boolean;
  readonly brands?: readonly { readonly brand: string }[];
  getHighEntropyValues?(hints: string[]): Promise<HighEntropyValues>;
}

type Navigated = Navigator & {
  readonly userAgentData?: ClientHints;
  readonly brave?: { isBrave?(): Promise<boolean> };
};

/** The synchronous part of detection; Brave answers later (see `refine`). */
export function readNavigator(nav: Navigator): DetectionInput {
  const { userAgentData: hints } = nav as Navigated;
  return {
    userAgent: nav.userAgent,
    maxTouchPoints: nav.maxTouchPoints ?? 0,
    ...(hints?.platform !== undefined && { uaDataPlatform: hints.platform }),
    ...(hints?.mobile !== undefined && { uaDataMobile: hints.mobile }),
    ...(hints?.brands && { uaBrands: hints.brands.map((entry) => entry.brand) }),
  };
}

export interface Refinement {
  readonly arch?: Arch;
  readonly browser?: BrowserId;
}

/** What only asynchronous APIs know: the real processor (Chromium client hints) and Brave. */
export async function refine(nav: Navigator): Promise<Refinement> {
  const { userAgentData: hints, brave } = nav as Navigated;
  const refinement: { arch?: Arch; browser?: BrowserId } = {};
  try {
    const values = await hints?.getHighEntropyValues?.(["architecture", "bitness"]);
    const arch = values && archFromHints(values);
    if (arch) refinement.arch = arch;
  } catch {
    // Not offered, or refused: the user agent's guess stands.
  }
  try {
    if ((await brave?.isBrave?.()) === true) refinement.browser = "brave";
  } catch {
    // Not Brave.
  }
  return refinement;
}
