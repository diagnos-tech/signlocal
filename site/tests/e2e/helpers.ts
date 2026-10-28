// Shared by the end-to-end specs: user agents, the pages of the site, and wizard URLs.

import type { Page } from "@playwright/test";
import { LOCALES } from "../../src/i18n/locales.ts";

export const USER_AGENTS = {
  windowsChrome:
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
  macSafari:
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Safari/605.1.15",
  linuxFirefox: "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0",
  iPhone:
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Mobile/15E148 Safari/604.1",
  android:
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36",
  chromeOs:
    "Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
  unknown: "curl/8.4.0",
} as const;

/** The pages every language has, as paths below the language folder. */
export const TRANSLATED_PAGES = [
  "",
  "download.html",
  "test/",
  "activate/",
  "compatibility.html",
  "help.html",
  "privacy.html",
] as const;
export const ENGLISH_PAGES = ["developers.html", "notices.html"] as const;

export function allPaths(): string[] {
  const translated = LOCALES.flatMap((locale) =>
    TRANSLATED_PAGES.map((page) => `${locale.folder}/${page}`),
  );
  return [...translated, ...ENGLISH_PAGES.map((page) => `/${page}`)];
}

export interface WizardLink {
  os?: string;
  arch?: string;
  distro?: string;
  browser?: string;
  cert?: string;
  device?: string;
  step?: string;
  fake?: string;
}

export function wizardUrl(link: WizardLink = {}, folder = ""): string {
  const { fake, ...choices } = link;
  const params = new URLSearchParams(Object.entries(choices) as [string, string][]);
  if (fake) params.set("sdk-fake", fake);
  const query = params.toString();
  return `${folder}/download.html${query ? `?${query}` : ""}`;
}

/** The names of the result cards showing, in page order. */
export async function visibleCards(page: Page): Promise<string[]> {
  return page
    .locator("[data-card]:visible")
    .evaluateAll((cards) => cards.map((card) => (card as HTMLElement).dataset.card ?? ""));
}

/** The four result combinations the accessibility and recipe specs use. */
export const RESULTS: ReadonlyArray<{ name: string; link: WizardLink; agent: string }> = [
  {
    name: "windows-chrome-token",
    link: { os: "windows", arch: "x64", browser: "chrome", cert: "token", step: "result" },
    agent: USER_AGENTS.windowsChrome,
  },
  {
    name: "macos-safari-file",
    link: { os: "macos", browser: "safari", cert: "file", step: "result" },
    agent: USER_AGENTS.macSafari,
  },
  {
    name: "linux-deb-firefox-card",
    link: {
      os: "linux",
      arch: "x64",
      distro: "deb",
      browser: "firefox",
      cert: "card",
      step: "result",
    },
    agent: USER_AGENTS.linuxFirefox,
  },
  {
    name: "linux-other-edge-unsure",
    link: {
      os: "linux",
      arch: "arm64",
      distro: "other",
      browser: "edge",
      cert: "unsure",
      step: "result",
    },
    agent: USER_AGENTS.linuxFirefox,
  },
];
