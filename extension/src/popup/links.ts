/**
 * Where the popup sends people: pages of the project's site (project.toml's
 * homepage), so the popup itself never needs network access.
 */

import { HOMEPAGE } from "../generated";

/** Desktop OS the popup can name, from `runtime.getPlatformInfo().os`. */
export type Platform = "win" | "mac" | "linux";

/** Section of the download page per OS. */
const DOWNLOAD_SECTION: Readonly<Record<Platform, string>> = {
  win: "windows",
  mac: "macos",
  linux: "linux",
};

// TODO(gustavo): Microsoft Store / Mac App Store links once the app is published there.
/** The download page, opened at `os`'s section. */
export function downloadUrl(os: Platform): string {
  return `${HOMEPAGE}download.html#${DOWNLOAD_SECTION[os]}`;
}

/**
 * Where people finish setting up an installed app (docs/ux.md §9): the page
 * opens the app through its URL scheme and follows the extension's
 * announcement live, which a popup cannot do (it closes on the prompt).
 */
export const ACTIVATE_URL = `${HOMEPAGE}activate/`;

/** The site's end-to-end check: finds the app and signs a harmless sample text. */
export const TEST_URL = `${HOMEPAGE}test/`;

/** The privacy notice. */
export const PRIVACY_URL = `${HOMEPAGE}privacy.html`;
