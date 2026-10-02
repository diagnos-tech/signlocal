import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach } from "vitest";

export const FAKE = join(import.meta.dirname, "fixtures", "fake-websign.mjs");

export const APP = {
  version: "1.4.0",
  protocols: { min: 1, max: 1 },
  os: "linux",
  arch: "x86_64",
  channel: "direct",
};

export const CERTIFICATE = {
  der: "AQID",
  chain: [],
  fingerprint: "ab".repeat(32),
  displayName: "Ana Beatriz Souza",
  issuerName: "AC Test",
  notBefore: 1741000000,
  notAfter: 1792000000,
  key: { type: "RSA", bits: 2048 },
  algorithms: ["RSASSA-PKCS1-v1_5"],
  profile: { icpBrasil: "A3", keyStorage: "hardware" },
};

export type Action = Record<string, unknown>;
export interface Rule {
  on: string;
  where?: Record<string, unknown>;
  repeat?: boolean;
  do: Action[];
}
export interface Scenario {
  rules: Rule[];
  ignoreStdinEnd?: boolean;
}

export const send = (message: Record<string, unknown>): Action => ({ send: message });

/** The app's happy-path answer to `hello`. */
export const helloRule = (protocol = 1): Rule => ({
  on: "hello",
  do: [send({ type: "hello", protocol, app: APP })],
});

const directories: string[] = [];
afterEach(() => {
  for (const directory of directories.splice(0))
    rmSync(directory, { recursive: true, force: true });
  delete process.env.WEBSIGN_FAKE_SCENARIO;
});

export interface Fake {
  /** Path to pass as `executable`. */
  readonly executable: string;
  /** Messages the fake read from the client, in order. */
  received(): Array<Record<string, unknown>>;
  /** Pid of the fake child (once started). */
  pid(): number;
  argv(): string[];
}

/** Installs a scenario for the next `websign connect` this process spawns. */
export function fake(scenario: Scenario): Fake {
  const directory = mkdtempSync(join(tmpdir(), "websign-fake-"));
  directories.push(directory);
  const log = join(directory, "log");
  const pidFile = join(directory, "pid");
  const argvFile = join(directory, "argv");
  const file = join(directory, "scenario.json");
  writeFileSync(log, "");
  writeFileSync(file, JSON.stringify({ ...scenario, log, pidFile, argvFile }));
  process.env.WEBSIGN_FAKE_SCENARIO = file;
  return {
    executable: FAKE,
    received: () =>
      readFileSync(log, "utf8")
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line)),
    pid: () => Number(readFileSync(pidFile, "utf8")),
    argv: () => JSON.parse(readFileSync(argvFile, "utf8")),
  };
}

export function isAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/** Polls until `check` is true; fails the test after `ms`. */
export async function eventually(check: () => boolean, ms = 3000): Promise<void> {
  const deadline = Date.now() + ms;
  while (!check()) {
    if (Date.now() > deadline) throw new Error("condition not met in time");
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}
