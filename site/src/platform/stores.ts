// Where each browser gets the extension: a store listing once project.toml names one, the
// developer install (a zip the person loads by hand) until then. Filling an ID in project.toml
// switches that browser to its store without any other change.

import type { BrowserId, Channel } from "./types.ts";

export interface StoreIds {
  readonly chromeWebStoreId: string;
  readonly edgeAddonsId: string;
  /** Empty until the listing is published: an unclaimed slug can be registered by anyone. */
  readonly firefoxAmoSlug: string;
}

export type StoreName = "Chrome Web Store" | "Edge Add-ons" | "Firefox Add-ons";

export interface ExtensionSource {
  readonly channel: Channel;
  /** Set for the `store` channel. */
  readonly store?: { readonly name: StoreName; readonly url: string };
  /** Edge using the Chrome Web Store: it asks whether to allow other stores. */
  readonly edgeUsesChromeStore: boolean;
  /** Opera needs the "Install Chrome Extensions" add-on first. */
  readonly operaNeedsAddon: boolean;
}

const CHROME_STORE = "https://chromewebstore.google.com/detail/";
const EDGE_STORE = "https://microsoftedge.microsoft.com/addons/detail/";
const AMO = "https://addons.mozilla.org/firefox/addon/";

const DEV: ExtensionSource = { channel: "dev", edgeUsesChromeStore: false, operaNeedsAddon: false };

export function extensionSource(browser: BrowserId, ids: StoreIds): ExtensionSource {
  const chrome = ids.chromeWebStoreId
    ? ({ name: "Chrome Web Store", url: `${CHROME_STORE}${ids.chromeWebStoreId}` } as const)
    : undefined;
  switch (browser) {
    case "chrome":
    case "brave":
    case "chromium":
      return chrome ? { ...DEV, channel: "store", store: chrome } : DEV;
    case "opera":
      return chrome ? { ...DEV, channel: "store", store: chrome, operaNeedsAddon: true } : DEV;
    case "edge":
      if (ids.edgeAddonsId) {
        const store = { name: "Edge Add-ons", url: `${EDGE_STORE}${ids.edgeAddonsId}` } as const;
        return { ...DEV, channel: "store", store };
      }
      return chrome ? { ...DEV, channel: "store", store: chrome, edgeUsesChromeStore: true } : DEV;
    case "firefox":
      return ids.firefoxAmoSlug
        ? {
            ...DEV,
            channel: "store",
            store: { name: "Firefox Add-ons", url: `${AMO}${ids.firefoxAmoSlug}/` },
          }
        : DEV;
    case "safari":
      return DEV;
  }
}
