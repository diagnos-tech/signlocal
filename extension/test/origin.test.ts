import { describe, expect, it, vi } from "vitest";
import { webContext } from "../src/background/origin";
import content from "../src/entrypoints/content";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));

const top = "https://top.example.org/page?x=1#h";

const LOOPBACK = [
  "http://localhost:3000/",
  "http://127.0.0.1:8080/",
  "http://[::1]:9000/",
  "http://app.localhost/",
  "http://a.b.localhost:5173/",
];

/** Whether a content-script match pattern (`scheme://host/*`) reaches `url`. */
function reaches(pattern: string, url: URL): boolean {
  const [, scheme, host] = /^([a-z*]+):\/\/([^/]+)\/\*$/.exec(pattern) ?? [];
  if (scheme === undefined || host === undefined || `${scheme}:` !== url.protocol) return false;
  if (host === "*") return true;
  if (host.startsWith("*.")) {
    const base = host.slice(2);
    return url.hostname === base || url.hostname.endsWith(`.${base}`);
  }
  return url.hostname === host;
}

describe("webContext", () => {
  it("returns serialized origins for https frame and tab", () => {
    expect(webContext("https://app.example.com/sign?a=b", top)).toEqual({
      origin: "https://app.example.com",
      topOrigin: "https://top.example.org",
    });
  });

  it("drops default ports and keeps explicit ones", () => {
    expect(webContext("https://app.example.com:443/x", "https://app.example.com:8443/")).toEqual({
      origin: "https://app.example.com",
      topOrigin: "https://app.example.com:8443",
    });
  });

  it.each(LOOPBACK)("accepts http on the loopback context %s", (url) => {
    const origin = new URL(url).origin;
    expect(webContext(url, url)).toEqual({ origin, topOrigin: origin });
  });

  it.each([
    "http://example.com/",
    "http://localhost.evil.com/",
    "http://128.0.0.1/",
    "http://127.5.6.7/",
    "http://192.168.0.10/",
    "file:///home/user/doc.html",
    "data:text/html,<p>x</p>",
    "chrome-extension://abcdefghijklmnopabcdefghijklmnop/page.html",
    "moz-extension://uuid/page.html",
    "about:blank",
    "javascript:alert(1)",
    "blob:https://app.example.com/uuid",
    "ftp://example.com/",
  ])("refuses the insecure or non-web frame %s", (url) => {
    expect(webContext(url, top)).toBeNull();
  });

  it("refuses when the top-level tab is insecure even if the frame is secure", () => {
    expect(webContext("https://app.example.com/", "http://example.com/")).toBeNull();
  });

  it("refuses a missing or unparsable URL", () => {
    expect(webContext(undefined, top)).toBeNull();
    expect(webContext("https://app.example.com/", undefined)).toBeNull();
    expect(webContext("not a url", top)).toBeNull();
    expect(webContext("", "")).toBeNull();
  });
});

describe("webContext and the content-script matches", () => {
  it.each(LOOPBACK)("the content script runs where http is accepted: %s", (url) => {
    const matches = content.matches as string[];
    expect(matches.some((pattern) => reaches(pattern, new URL(url)))).toBe(true);
  });

  it.each(["http://127.5.6.7/", "http://example.com/"])(
    "no match pattern reaches the refused http page %s",
    (url) => {
      const matches = content.matches as string[];
      expect(webContext(url, url)).toBeNull();
      expect(matches.some((pattern) => reaches(pattern, new URL(url)))).toBe(false);
    },
  );
});
