/**
 * Where `websign` may live, as a pure function of the environment so every
 * operating system's search order is testable from any host.
 */

import { accessSync, constants, lstatSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { posix, win32 } from "node:path";

export interface Environment {
  readonly env: Readonly<Record<string, string | undefined>>;
  readonly platform: NodeJS.Platform;
  readonly home: string;
  readonly isExecutable: (path: string) => boolean;
}

/** The running process's environment. */
export function currentEnvironment(): Environment {
  return {
    env: process.env,
    platform: process.platform,
    home: homedir(),
    isExecutable,
  };
}

/**
 * On Windows anything that is not a directory counts: the MSIX alias in
 * `WindowsApps` is a reparse point that `stat` may report as a link or refuse
 * to follow, yet `CreateProcess` starts it. Elsewhere the execute bit decides.
 */
function isExecutable(path: string): boolean {
  try {
    if (process.platform === "win32") return !windowsStat(path).isDirectory();
    const stats = statSync(path);
    if (!stats.isFile()) return false;
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function windowsStat(path: string) {
  try {
    return statSync(path);
  } catch {
    return lstatSync(path);
  }
}

/** First existing executable in the documented order, or `undefined`. */
export function search(environment: Environment): string | undefined {
  const override = environment.env.WEBSIGN_EXECUTABLE;
  const ordered = [
    ...(override ? [override] : []),
    ...onPath(environment),
    ...installLocations(environment),
  ];
  return ordered.find(environment.isExecutable);
}

function onPath({ env, platform }: Environment): string[] {
  const windows = platform === "win32";
  const path = windows ? win32 : posix;
  const variable = windows ? Object.keys(env).find((k) => k.toLowerCase() === "path") : "PATH";
  const value = variable === undefined ? undefined : env[variable];
  if (!value) return [];
  const name = windows ? "websign.exe" : "websign";
  return value
    .split(path.delimiter)
    .map((directory) => directory.replaceAll('"', ""))
    .filter((directory) => directory !== "")
    .map((directory) => path.join(directory, name));
}

function installLocations({ env, platform, home }: Environment): string[] {
  switch (platform) {
    case "win32": {
      const local = env.LOCALAPPDATA;
      if (!local) return [];
      return [
        win32.join(local, "Programs", "WebeSign", "websign.exe"),
        win32.join(local, "Microsoft", "WindowsApps", "websign.exe"),
      ];
    }
    case "darwin":
      return [
        "/Applications/WebeSign.app/Contents/MacOS/websign",
        posix.join(home, "Applications", "WebeSign.app", "Contents", "MacOS", "websign"),
        posix.join(home, ".local", "bin", "websign"),
      ];
    default:
      return ["/usr/bin/websign", posix.join(home, ".local", "bin", "websign")];
  }
}
