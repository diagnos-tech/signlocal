/**
 * The manifest fields WXT does not derive from the entrypoints, as a pure
 * function of the target so tests can pin every variant.
 *
 * Channels (docs/architecture/packaging-and-release.md): `direct` builds are loaded
 * unpacked from the release zip, so Chromium ones carry `project.toml`'s
 * `dev_key`: it pins the extension ID to `dev_id`, the ID the native host's
 * manifest allows. `store` builds must not carry it (the stores assign and
 * sign the ID). Only `direct` exists today, so it is the default; store
 * uploads set `WEBSIGN_CHANNEL=store`.
 */

import { DEV_KEY, FIREFOX_ID } from "../src/generated/project";

/** How the build reaches people. */
export type Channel = "direct" | "store";

/** What the manifest depends on. */
export interface Target {
  readonly browser: string;
  /** Vite mode: "development" under `wxt` (dev server), "production" for builds. */
  readonly mode: string;
  readonly channel: Channel;
}

/**
 * Extension pages load only their own files: no remote code, no inline
 * script, no network. Omitted under the dev server, which serves the popup
 * from localhost with hot reload.
 */
export const EXTENSION_PAGES_CSP =
  "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; " +
  "object-src 'none'; base-uri 'none'; form-action 'none'";

/** The channel named by `WEBSIGN_CHANNEL`; unknown values fail the build loudly. */
export function channelFromEnv(value: string | undefined): Channel {
  if (value === undefined || value === "" || value === "direct") return "direct";
  if (value === "store") return "store";
  throw new Error(`WEBSIGN_CHANNEL must be "direct" or "store", not "${value}"`);
}

function isChromium(browser: string): boolean {
  return browser !== "firefox" && browser !== "safari";
}

/** Store and management-page icons; the same art for every browser. */
const ICONS = Object.fromEntries(
  [16, 32, 48, 96, 128].map((size) => [size, `icons/icon-${size}.png`]),
);

/**
 * Toolbar icons: 16/19/32/38 px are the sizes Chromium and Firefox pick from
 * at 1x and 2x (19 and 38 are Chrome's legacy sizes). Safari draws toolbar
 * icons as monochrome templates, so it gets the black-on-transparent set.
 */
function toolbarIcons(browser: string): Record<string, string> {
  if (browser === "safari") {
    return { 16: "icons/template-16.png", 32: "icons/template-32.png" };
  }
  return Object.fromEntries([16, 19, 32, 38].map((size) => [size, `icons/icon-${size}.png`]));
}

/** Manifest fields for `target`, merged by WXT over what it derives itself. */
export function manifestFor(target: Target): Record<string, unknown> {
  const { browser, mode, channel } = target;
  const pinId = isChromium(browser) && (mode === "development" || channel === "direct");
  return {
    name: "__MSG_extension_name__",
    description: "__MSG_extension_description__",
    default_locale: "en",
    permissions: ["nativeMessaging"],
    minimum_chrome_version: "121",
    icons: ICONS,
    action: { default_icon: toolbarIcons(browser) },
    ...(pinId ? { key: DEV_KEY } : {}),
    ...(mode === "development"
      ? {}
      : { content_security_policy: { extension_pages: EXTENSION_PAGES_CSP } }),
    ...(browser === "firefox"
      ? {
          browser_specific_settings: {
            gecko: {
              id: FIREFOX_ID,
              strict_min_version: "121.0",
              data_collection_permissions: { required: ["none"] },
            },
          },
        }
      : {}),
  };
}
