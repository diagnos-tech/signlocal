/**
 * The macOS crash report of an app process killed by a signal. A native
 * trap (EXC_BREAKPOINT from a system framework) prints nothing to stderr and
 * logs nothing; ReportCrash writes the only account of it, a second or so
 * after the process died, to ~/Library/Logs/DiagnosticReports.
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import { homedir, platform } from "node:os";
import { basename, join } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const REPORTS = join(homedir(), "Library", "Logs", "DiagnosticReports");
/** ReportCrash usually needs a second or two; give it more on a busy runner. */
const WAIT_MS = 15_000;
const FRAMES = 30;

/**
 * The exception, the crash messages and the crashed thread's frames of the
 * newest report of `program` written after `since` (ms since the epoch);
 * empty off macOS.
 */
export async function crashSummary(program: string, since: number): Promise<string> {
  if (platform() !== "darwin") return "";
  const name = basename(program);
  for (let waited = 0; waited <= WAIT_MS; waited += 500) {
    const report = newestReport(name, since);
    if (report !== undefined) return `crash report ${report}:\n${summarize(report)}`;
    await delay(500);
  }
  return `no crash report of ${name} in ${REPORTS}`;
}

function newestReport(name: string, since: number): string | undefined {
  try {
    return readdirSync(REPORTS)
      .filter((file) => file.startsWith(`${name}-`) && /\.(ips|crash)$/.test(file))
      .map((file) => join(REPORTS, file))
      .map((path) => ({ path, time: statSync(path).mtimeMs }))
      .filter((report) => report.time >= since - 1_000)
      .sort((a, b) => b.time - a.time)[0]?.path;
  } catch {
    return undefined;
  }
}

interface Frame {
  imageIndex?: number;
  imageOffset?: number;
  symbol?: string;
}

interface IpsBody {
  exception?: unknown;
  termination?: unknown;
  asi?: unknown;
  faultingThread?: number;
  threads?: { name?: string; queue?: string; frames?: Frame[] }[];
  usedImages?: { name?: string }[];
}

/** An `.ips` is a JSON header line followed by a JSON body; anything else is shown raw. */
function summarize(path: string): string {
  const text = readFileSync(path, "utf8");
  try {
    const body = JSON.parse(text.slice(text.indexOf("\n") + 1)) as IpsBody;
    const thread = body.threads?.[body.faultingThread ?? -1];
    const frames = (thread?.frames ?? []).slice(0, FRAMES).map((frame) => {
      const image = body.usedImages?.[frame.imageIndex ?? -1]?.name ?? "?";
      return `  ${image}  ${frame.symbol ?? `+0x${(frame.imageOffset ?? 0).toString(16)}`}`;
    });
    return [
      `exception: ${JSON.stringify(body.exception)}`,
      `termination: ${JSON.stringify(body.termination)}`,
      `messages: ${JSON.stringify(body.asi)}`,
      `crashed thread ${body.faultingThread} (${thread?.name ?? thread?.queue ?? "unnamed"}):`,
      ...frames,
    ].join("\n");
  } catch {
    return text.split("\n").slice(0, 80).join("\n");
  }
}
