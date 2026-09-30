import { describe, expect, it } from "vitest";
import { webContext } from "../src/background/origin";

const top = "https://top.example.org/page?x=1#h";

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

  it.each([
    "http://localhost:3000/",
    "http://127.0.0.1:8080/",
    "http://127.5.6.7/",
    "http://[::1]:9000/",
    "http://app.localhost/",
  ])("accepts http on the loopback context %s", (url) => {
    const origin = new URL(url).origin;
    expect(webContext(url, url)).toEqual({ origin, topOrigin: origin });
  });

  it.each([
    "http://example.com/",
    "http://localhost.evil.com/",
    "http://128.0.0.1/",
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
