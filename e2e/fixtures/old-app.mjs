/**
 * A fake old app for the AppOutdated scenario: a native messaging host that
 * answers `hello` as a build older than the extension's MIN_APP_VERSION
 * would, and ignores everything else until the browser hangs up. Test-only:
 * `lib/old-app.ts` registers it in place of the app, in a throwaway profile.
 *
 * Native messaging frames are a 32-bit little-endian length and UTF-8 JSON.
 */

import { arch, platform } from "node:os";

/** Older than any MIN_APP_VERSION the extension can require. */
const OLD_VERSION = "0.0.1";
const OS = { win32: "windows", darwin: "macos" }[platform()] ?? "linux";

let pending = Buffer.alloc(0);

function reply(message) {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  const length = Buffer.alloc(4);
  length.writeUInt32LE(body.length);
  process.stdout.write(Buffer.concat([length, body]));
}

function answer(message) {
  if (message?.type !== "hello") return;
  const protocol = message.protocols?.max ?? message.v;
  reply({
    v: message.v,
    id: message.id,
    type: "hello",
    app: {
      version: OLD_VERSION,
      protocols: { min: protocol, max: protocol },
      os: OS,
      arch: arch() === "arm64" ? "aarch64" : "x86_64",
      channel: "direct",
    },
    protocol,
  });
}

process.stdin.on("data", (chunk) => {
  pending = Buffer.concat([pending, chunk]);
  while (pending.length >= 4) {
    const length = pending.readUInt32LE(0);
    if (pending.length < 4 + length) return;
    const body = pending.subarray(4, 4 + length).toString("utf8");
    pending = pending.subarray(4 + length);
    try {
      answer(JSON.parse(body));
    } catch {
      // Not JSON: an old app would not understand it either.
    }
  }
});
process.stdin.on("end", () => process.exit(0));
