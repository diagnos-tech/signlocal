// The wizard's URL contract, documented for integrators on /developers.html:
//
//   /download.html?os=windows&arch=x64&browser=chrome&cert=token&device=safenet-etoken-5110&step=result
//
// Unknown parameters and invalid values are dropped silently, never an error page. The legacy
// hash of the extension popup (#windows, #chrome …) is understood too.

import { browserValidFor, needsArch, needsDistro } from "../platform/catalog.ts";
import {
  ARCH_IDS,
  BROWSER_IDS,
  CERT_IDS,
  type Choices,
  DISTRO_IDS,
  OS_IDS,
  STEPS,
  type Step,
} from "../platform/types.ts";

export type DeviceKind = "token" | "card" | "reader";
/** The kind of a devices.json id, or undefined when there is no such device. */
export type DeviceLookup = (id: string) => DeviceKind | undefined;

export interface UrlState {
  readonly choices: Choices;
  readonly step?: Step;
}

/** Whether a device of `kind` may be chosen for a certificate kind. */
export function deviceFits(kind: DeviceKind | undefined, cert: Choices["cert"]): boolean {
  if (kind === undefined || kind === "reader") return false;
  return cert === "unsure" || cert === kind;
}

const oneOf = <T extends string>(allowed: readonly T[], value: string | null): T | undefined =>
  allowed.find((candidate) => candidate === value);

/** Reads a query string and a legacy hash ("?os=…" and "#macos", with or without the marks). */
export function parseUrl(search: string, hash: string, lookup: DeviceLookup): UrlState {
  const params = new URLSearchParams(search);
  const legacy = hash.replace(/^#/, "");
  const choices: Choices = {};
  const os = oneOf(OS_IDS, params.get("os")) ?? oneOf(OS_IDS, legacy);
  if (os) choices.os = os;
  const arch = oneOf(ARCH_IDS, params.get("arch"));
  if (arch && (!os || needsArch(os))) choices.arch = arch;
  const distro = oneOf(DISTRO_IDS, params.get("distro"));
  if (distro && (!os || needsDistro(os))) choices.distro = distro;
  const browser = oneOf(BROWSER_IDS, params.get("browser")) ?? oneOf(BROWSER_IDS, legacy);
  if (browser && (!os || browserValidFor(browser, os))) choices.browser = browser;
  const cert = oneOf(CERT_IDS, params.get("cert"));
  if (cert) choices.cert = cert;
  const device = params.get("device");
  if (device && cert && deviceFits(lookup(device), cert)) choices.device = device;
  const step = oneOf(STEPS, params.get("step"));
  return step ? { choices, step } : { choices };
}

/** Query string without the "?", parameters in the documented order, empty ones omitted. */
export function serializeUrl(choices: Choices, step?: Step): string {
  const params = new URLSearchParams();
  const { os, arch, distro, browser, cert, device } = choices;
  if (os) params.set("os", os);
  if (arch && os && needsArch(os)) params.set("arch", arch);
  if (distro && os && needsDistro(os)) params.set("distro", distro);
  if (browser) params.set("browser", browser);
  if (cert) params.set("cert", cert);
  if (device && cert) params.set("device", device);
  if (step) params.set("step", step);
  return params.toString();
}
