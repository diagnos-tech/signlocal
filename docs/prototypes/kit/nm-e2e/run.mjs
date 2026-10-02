#!/usr/bin/env node
// End-to-end check of the whole browser path, in a real Chromium:
//
//   page -> content script -> service worker -> native messaging -> websign-probe
//
// Registers the probe as native host for a throwaway Chromium profile, loads
// the test extension, lets page.html ping/list/sign through it and judges the
// outcome together with the host's own log.
//
//   node run.mjs --probe <websign-probe> [--chromium <exe>] [--no-register] [--expect-sign]
//
// Prints `NM-E2E: PASS` or `NM-E2E: FAIL <reason>` (then the host log) and
// exits 0 or 1.

import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { readProject } from "../extension/project-config.mjs";
import { launchWithExtension } from "./lib/browser.mjs";
import { parseCli } from "./lib/cli.mjs";
import { hostLogPath, watchHostLog } from "./lib/host-log.mjs";
import { startPageServer } from "./lib/page-server.mjs";
import { judge } from "./lib/verdict.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const extensionDir = resolve(here, "..", "extension");
const RESULT_TIMEOUT_MS = 150_000;

async function main() {
  const options = parseCli(process.argv.slice(2));
  const project = readProject();
  const probe = resolve(options.probe);

  // The manifest and config.js must match project.toml before Chromium loads them.
  execFileSync(process.execPath, [join(extensionDir, "generate-manifest.mjs")], { stdio: "inherit" });

  const userDataDir = mkdtempSync(join(tmpdir(), "websign-nm-e2e-"));
  const hostLog = watchHostLog(hostLogPath(project.slug));
  const page = await startPageServer();
  let context;
  let result = null;
  const pageMessages = [];
  try {
    if (options.register) {
      execFileSync(probe, ["register", "--browser", "chromium", "--user-data-dir", userDataDir], {
        stdio: "inherit",
      });
    }
    context = await launchWithExtension({
      executablePath: options.chromium,
      userDataDir,
      extensionDir,
      headed: options.headed,
    });
    const tab = await context.newPage();
    tab.on("console", (message) => pageMessages.push(`console.${message.type()}: ${message.text()}`));
    tab.on("pageerror", (error) => pageMessages.push(`pageerror: ${error.message}`));
    await tab.goto(page.url);
    await tab.waitForFunction(() => window.__result?.done === true, null, { timeout: RESULT_TIMEOUT_MS });
    result = await tab.evaluate(() => window.__result);
  } catch (error) {
    result = { failure: String(error?.message ?? error) };
  } finally {
    // Closing the browser closes the host's stdin, which is how it should exit.
    await context?.close().catch(() => {});
    await page.close();
    rmSync(userDataDir, { recursive: true, force: true });
  }

  // Give the host a moment to write its last line after the browser closed.
  await new Promise((done) => setTimeout(done, 500));
  const log = hostLog.lines();
  const verdict = judge(result, log, { devId: project.devId, expectSign: options.expectSign });

  for (const warning of verdict.warnings) console.log(`NM-E2E: WARN ${warning}`);
  console.log(verdict.ok ? "NM-E2E: PASS" : `NM-E2E: FAIL ${verdict.reason}`);
  console.log(`--- host log (${hostLog.path}), this run ---`);
  console.log(log.length > 0 ? log.slice(-40).join("\n") : "(no host log lines: the host never started)");
  if (!verdict.ok) {
    console.log("--- page result ---");
    console.log(JSON.stringify(result, null, 2));
    if (pageMessages.length > 0) console.log(["--- page console ---", ...pageMessages].join("\n"));
  }
  process.exit(verdict.ok ? 0 : 1);
}

main().catch((error) => {
  console.log(`NM-E2E: FAIL ${error?.stack ?? error}`);
  process.exit(1);
});
