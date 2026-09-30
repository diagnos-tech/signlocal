/** Builders for protocol messages used across the suites. */

import type { AppEnvelope, AppInfo, HelloReply } from "../../src/generated";

/** An app description; `version` decides outdated/ready. */
export function appInfo(version = "1.4.0"): AppInfo {
  return { version, protocols: { min: 1, max: 1 }, os: "linux", arch: "x86_64", channel: "direct" };
}

/** The app's hello reply payload. */
export function helloReply(version = "1.4.0"): HelloReply {
  return { app: appInfo(version), protocol: 1 };
}

/** A full `hello` envelope answering the client message with `id`. */
export function helloEnvelope(id: string, version = "1.4.0"): AppEnvelope {
  return { v: 1, id, type: "hello", ...helloReply(version) };
}
