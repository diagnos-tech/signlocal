import { describe, expect, test } from "bun:test";
import { type DeviceKind, parseUrl, serializeUrl } from "../../src/wizard/url.ts";

const kinds: Record<string, DeviceKind> = {
  "safenet-etoken-5110": "token",
  "pt-cartao-de-cidadao": "card",
  "omnikey-3121": "reader",
};
const lookup = (id: string) => kinds[id];

describe("parseUrl", () => {
  test("reads every parameter of the contract", () => {
    const url =
      "?os=windows&arch=arm64&browser=edge&cert=token&device=safenet-etoken-5110&step=result";
    expect(parseUrl(url, "", lookup)).toEqual({
      choices: {
        os: "windows",
        arch: "arm64",
        browser: "edge",
        cert: "token",
        device: "safenet-etoken-5110",
      },
      step: "result",
    });
  });

  test("drops unknown parameters and invalid values silently", () => {
    expect(
      parseUrl("?os=beos&browser=netscape&cert=paper&step=nowhere&x=1&lang=de", "", lookup),
    ).toEqual({
      choices: {},
    });
  });

  test("arch only counts for Windows and Linux, distro only for Linux", () => {
    expect(parseUrl("?os=macos&arch=x64&distro=deb", "", lookup).choices).toEqual({ os: "macos" });
    expect(parseUrl("?os=linux&arch=arm64&distro=rpm", "", lookup).choices).toEqual({
      os: "linux",
      arch: "arm64",
      distro: "rpm",
    });
  });

  test("Safari is dropped when the system is not macOS", () => {
    expect(parseUrl("?os=windows&browser=safari", "", lookup).choices).toEqual({ os: "windows" });
    expect(parseUrl("?os=macos&browser=safari", "", lookup).choices).toEqual({
      os: "macos",
      browser: "safari",
    });
  });

  test("a device must exist and match the certificate kind", () => {
    expect(parseUrl("?cert=card&device=safenet-etoken-5110", "", lookup).choices).toEqual({
      cert: "card",
    });
    expect(parseUrl("?cert=card&device=pt-cartao-de-cidadao", "", lookup).choices.device).toBe(
      "pt-cartao-de-cidadao",
    );
    expect(parseUrl("?cert=unsure&device=safenet-etoken-5110", "", lookup).choices.device).toBe(
      "safenet-etoken-5110",
    );
    expect(parseUrl("?cert=token&device=omnikey-3121", "", lookup).choices).toEqual({
      cert: "token",
    });
    expect(parseUrl("?cert=file&device=safenet-etoken-5110", "", lookup).choices).toEqual({
      cert: "file",
    });
    expect(parseUrl("?device=safenet-etoken-5110", "", lookup).choices).toEqual({});
  });

  test("the legacy hash of the popup names a system or a browser", () => {
    expect(parseUrl("", "#macos", lookup).choices).toEqual({ os: "macos" });
    expect(parseUrl("", "#linux", lookup).choices).toEqual({ os: "linux" });
    expect(parseUrl("", "#firefox", lookup).choices).toEqual({ browser: "firefox" });
    expect(parseUrl("", "#nothing", lookup).choices).toEqual({});
  });

  test("the query wins over the hash", () => {
    expect(parseUrl("?os=linux", "#windows", lookup).choices).toEqual({ os: "linux" });
  });

  test("lang is ignored here: the boot script handles it", () => {
    expect(parseUrl("?lang=pt-BR&os=macos", "", lookup).choices).toEqual({ os: "macos" });
  });
});

describe("serializeUrl", () => {
  test("writes the parameters in the documented order and omits empty ones", () => {
    const choices = { cert: "token", browser: "chrome", os: "windows", arch: "x64" } as const;
    expect(serializeUrl(choices, "browser")).toBe(
      "os=windows&arch=x64&browser=chrome&cert=token&step=browser",
    );
    expect(serializeUrl({})).toBe("");
  });

  test("never writes parameters that do not apply", () => {
    expect(serializeUrl({ os: "macos", arch: "x64", distro: "deb" })).toBe("os=macos");
    expect(serializeUrl({ os: "windows", distro: "deb" })).toBe("os=windows");
    expect(serializeUrl({ device: "x" })).toBe("");
  });

  test("round-trips a full set of choices", () => {
    const choices = {
      os: "linux",
      arch: "arm64",
      distro: "other",
      browser: "firefox",
      cert: "unsure",
      device: "safenet-etoken-5110",
    } as const;
    const query = serializeUrl(choices, "result");
    expect(parseUrl(`?${query}`, "", lookup)).toEqual({ choices, step: "result" });
  });
});
