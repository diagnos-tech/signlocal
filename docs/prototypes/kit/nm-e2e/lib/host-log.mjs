// The native host's diagnostic log: the evidence of what happened between
// the browser and the host, since the browser discards the host's stderr.

import { existsSync, readFileSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

/** Same path the host computes: `<temp>/<slug>-probe-host.log`. */
export function hostLogPath(slug) {
  return join(tmpdir(), `${slug}-probe-host.log`);
}

/** Remembers where the log ended so a run reads only its own lines. */
export function watchHostLog(path) {
  const start = existsSync(path) ? statSync(path).size : 0;
  return {
    path,
    /** Lines the host wrote since `watchHostLog` was called. */
    lines() {
      if (!existsSync(path)) return [];
      const bytes = readFileSync(path);
      // The host truncates an oversized log at startup: read it all then.
      const from = bytes.length >= start ? start : 0;
      return bytes.subarray(from).toString("utf8").split("\n").filter(Boolean);
    },
  };
}
