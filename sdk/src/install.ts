import { lastAnnouncement } from "./channel.js";
import { CHROME_WEB_STORE_ID, EDGE_ADDONS_ID, FIREFOX_AMO_SLUG, HOMEPAGE } from "./project.js";

/** Browsers by family, in the words the extension and `navigator` use. */
type Store = "chrome" | "edge" | "firefox" | "other";

const FAMILY: Readonly<Record<string, Store>> = {
  chrome: "chrome",
  chromium: "chrome",
  brave: "chrome",
  opera: "chrome",
  vivaldi: "chrome",
  edge: "edge",
  firefox: "firefox",
};

/** The browser as the user agent tells it, for pages where the extension is not there to say. */
function fromNavigator(): string {
  const nav = (
    globalThis as { navigator?: Navigator & { userAgentData?: { brands?: { brand: string }[] } } }
  ).navigator;
  const brands = nav?.userAgentData?.brands?.map((b) => b.brand).join(" ") ?? "";
  const text = `${brands} ${nav?.userAgent ?? ""}`.toLowerCase();
  if (/\bedg(e|a|ios)?\b/.test(text)) return "edge";
  if (/firefox|fxios/.test(text)) return "firefox";
  if (/opr\/|opera/.test(text)) return "opera";
  if (text.includes("vivaldi")) return "vivaldi";
  return /chrome|chromium|crios/.test(text) ? "chrome" : "other";
}

/**
 * Where to send a person who lacks the extension: the store of the current
 * browser (Chrome Web Store, Edge Add-ons, Firefox AMO; Safari → the app's
 * download page), or the project's download page when unknown or while the
 * store listing does not exist yet. A listing is linked only once
 * project.toml names it: an unpublished AMO slug, for one, can be claimed
 * by anyone, so guessing it would send people to a stranger's add-on.
 *
 * The browser comes from the extension's last announcement when there is
 * one (it knows better), else from the user agent.
 */
export function installUrl(): string {
  const browser = lastAnnouncement()?.extension.browser ?? fromNavigator();
  const store = FAMILY[browser] ?? "other";
  if (store === "chrome" && CHROME_WEB_STORE_ID) {
    return `https://chromewebstore.google.com/detail/${CHROME_WEB_STORE_ID}`;
  }
  if (store === "edge" && EDGE_ADDONS_ID) {
    return `https://microsoftedge.microsoft.com/addons/detail/${EDGE_ADDONS_ID}`;
  }
  if (store === "firefox" && FIREFOX_AMO_SLUG) {
    return `https://addons.mozilla.org/firefox/addon/${FIREFOX_AMO_SLUG}/`;
  }
  return `${HOMEPAGE}download.html`;
}
