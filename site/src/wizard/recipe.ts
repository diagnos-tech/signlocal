// The ordered list of cards the result step shows for a set of choices. It is the documented
// truth for the tests: the cards visible on the page must equal it.

import type { Choices } from "../platform/types.ts";

export type ResultCard =
  | "install"
  | "extension"
  | "connect"
  | "certificate"
  | "test"
  | "no-release";

/**
 * Safari comes with the app, so its order is: install, open signlocal once, turn the
 * extension on. Every other browser: install, extension, connect. With no release published
 * there is nothing to install: the page says so, then still prepares the certificate.
 */
export function recipe(choices: Choices, releaseAvailable: boolean): readonly ResultCard[] {
  if (!releaseAvailable) return ["no-release", "certificate"];
  return choices.browser === "safari"
    ? ["install", "connect", "extension", "certificate", "test"]
    : ["install", "extension", "connect", "certificate", "test"];
}

/**
 * The install method a system starts on. Fresh Ubuntu and Debian desktops lack `curl`, which
 * the installer script needs, so the package download (which `apt` or `dnf` handles) opens first.
 */
export function defaultInstallTab(choices: Choices): "auto" | "manual" {
  return choices.os === "linux" && (choices.distro === "deb" || choices.distro === "rpm")
    ? "manual"
    : "auto";
}
