/**
 * Puts the fake old app (`fixtures/old-app.mjs`) where the real one was
 * registered in a throwaway profile: the registration the app wrote stays
 * (same host name, same allowed extension, same Windows registry key), only
 * the manifest's `path` changes to a launcher that runs the script with
 * this Node.js. Nothing outside the profile is touched.
 */

import { chmodSync, existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { platform } from "node:os";
import { join } from "node:path";

import { NATIVE_HOST } from "../../extension/src/generated/project.ts";
import { REPO } from "./environment.ts";

const SCRIPT = join(REPO, "e2e", "fixtures", "old-app.mjs");

/** Points every manifest of the app in `profile` at the fake old app. */
export function installOldApp(profile: string): void {
  const launcher = writeLauncher(profile);
  const manifests = manifestFiles(profile);
  if (manifests.length === 0) throw new Error(`no ${NATIVE_HOST} manifest in ${profile}`);
  for (const file of manifests) {
    const manifest = JSON.parse(readFileSync(file, "utf8")) as { path: string };
    writeFileSync(file, JSON.stringify({ ...manifest, path: launcher }));
  }
}

/** Manifests `websign register` wrote: in the profile (Windows) or its NativeMessagingHosts. */
function manifestFiles(profile: string): string[] {
  return [profile, join(profile, "NativeMessagingHosts")]
    .filter((dir) => existsSync(dir))
    .flatMap((dir) => readdirSync(dir).map((file) => join(dir, file)))
    .filter((file) => file.endsWith(".json"))
    .filter((file) => {
      try {
        return (JSON.parse(readFileSync(file, "utf8")) as { name?: unknown }).name === NATIVE_HOST;
      } catch {
        return false;
      }
    });
}

/** A launcher the browser can start (`.cmd` on Windows, a shell script elsewhere). */
function writeLauncher(profile: string): string {
  const node = process.execPath;
  if (platform() === "win32") {
    const launcher = join(profile, "old-app.cmd");
    writeFileSync(launcher, `@"${node}" "${SCRIPT}" %*\r\n`);
    return launcher;
  }
  const launcher = join(profile, "old-app.sh");
  writeFileSync(launcher, `#!/bin/sh\nexec "${node}" "${SCRIPT}" "$@"\n`);
  chmodSync(launcher, 0o755);
  return launcher;
}
