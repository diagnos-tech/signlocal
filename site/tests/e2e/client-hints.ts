// The client hints a real browser with a given user agent offers. Chromium-based browsers expose
// navigator.userAgentData (brands, platform, mobile, high-entropy values); Safari and Firefox
// expose none. Playwright's `userAgent` option changes only the user agent string and keeps the
// real browser's brands, so without this Playwright's own Chromium reads as "Chromium" under a
// Chrome user agent, and as Chromium-based under Safari's.

export interface ClientHints {
  readonly brands: readonly { readonly brand: string; readonly version: string }[];
  readonly mobile: boolean;
  readonly platform: string;
  readonly architecture: string;
  readonly bitness: string;
}

function brandOf(userAgent: string): string {
  if (/Edg\//.test(userAgent)) return "Microsoft Edge";
  if (/OPR\//.test(userAgent)) return "Opera";
  return "Google Chrome";
}

function platformOf(userAgent: string): string {
  if (/Android/.test(userAgent)) return "Android";
  if (/CrOS/.test(userAgent)) return "Chrome OS";
  if (/Windows/.test(userAgent)) return "Windows";
  if (/Macintosh|Mac OS X/.test(userAgent)) return "macOS";
  return "Linux";
}

/** What a browser with this user agent tells a page through client hints; null: no such API. */
export function clientHintsFor(userAgent: string): ClientHints | null {
  const chrome = /Chrome\/(\d+)/.exec(userAgent);
  if (!chrome) return null;
  const version = chrome[1] ?? "0";
  return {
    brands: [
      { brand: brandOf(userAgent), version },
      { brand: "Chromium", version },
      { brand: "Not?A_Brand", version: "99" },
    ],
    mobile: /Mobile/.test(userAgent),
    platform: platformOf(userAgent),
    architecture: /arm|aarch64/i.test(userAgent) ? "arm" : "x86",
    bitness: /Win64|x64|x86_64|aarch64|arm64/i.test(userAgent) ? "64" : "32",
  };
}

/** Runs in the page (Playwright serializes it): self-contained, no imports or closures. */
export function installClientHints(hints: ClientHints | null): void {
  const data = hints && {
    brands: hints.brands,
    mobile: hints.mobile,
    platform: hints.platform,
    getHighEntropyValues: async () => ({ ...hints }),
    toJSON: () => ({ brands: hints.brands, mobile: hints.mobile, platform: hints.platform }),
  };
  Object.defineProperty(Navigator.prototype, "userAgentData", {
    configurable: true,
    get: () => data ?? undefined,
  });
}
