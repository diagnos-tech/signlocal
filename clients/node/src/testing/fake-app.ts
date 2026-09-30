import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { WebSign } from "../client.js";
import type { ErrorCode, Certificate as WireCertificate } from "../generated/index.js";
import type { ConnectOptions } from "../types.js";
import { SAMPLE_CERTIFICATE } from "./sample-certificate.js";

const PROCESS = fileURLToPath(new URL("./app-process.js", import.meta.url));

/**
 * A certificate as the app sends it over the wire: `der`/`chain` Base64,
 * validity in Unix seconds. `connect()` decodes it into the public
 * `Certificate`. Start from {@link SAMPLE_CERTIFICATE} and override fields.
 */
export type FakeCertificate = WireCertificate;

/** How the fake app should fail instead of answering. */
export interface FakeFailure {
  /** The protocol code the app reports, e.g. `UserCancelled` or `PinLocked`. */
  readonly code: ErrorCode;
  readonly message?: string;
  /**
   * `choose` (default): at the certificate window, before `prepare` runs.
   * `confirm`: after `prepare` returned, like a wrong PIN or a removed token.
   */
  readonly when?: "choose" | "confirm";
}

/** What the fake app offers and how it behaves. Every field is optional. */
export interface FakeAppOptions {
  /** Certificates the person "has"; default one ECDSA P-256 {@link SAMPLE_CERTIFICATE}. */
  readonly certificates?: readonly FakeCertificate[];
  /** Fail instead of signing, to test your error handling. */
  readonly failWith?: FakeFailure;
  /** Base64 returned as `signature`; default the digest itself (NOT a valid signature). */
  readonly signature?: string;
  /** What `status()` reports as `remembered`; default `false`. */
  readonly remembered?: boolean;
}

/** A fake `websign connect` and what it heard. */
export interface FakeApp {
  /** Pass to `WebSign.connect(...)` (or call {@link FakeApp.connect}). */
  readonly options: ConnectOptions;
  /** Starts a real connection to the fake; closed by {@link FakeApp.close}. */
  connect(extra?: ConnectOptions): Promise<WebSign>;
  /** The messages the client sent, in order (`hello`, `sign.begin`, ...). */
  requests(): Array<Record<string, unknown>>;
  /** Closes every connection made through {@link FakeApp.connect} and deletes the scratch files. */
  close(): Promise<void>;
}

/**
 * A stand-in for the WebeSign app, so tests of your program need neither the
 * app nor a person. It is a real child process speaking the real protocol:
 * your code runs unchanged through `WebSign.connect`. It answers `hello`,
 * `status`, `choose`, `sign` and `diagnostics.open`, enforces the same
 * digest-length rule as the app, and answers `sign` with the digest itself as
 * `signature` (so it never verifies: do not use it to test signature checks).
 *
 * @example
 * ```ts
 * import { fakeApp } from "@websign/desktop/testing";
 *
 * const app = fakeApp();
 * const websign = await app.connect();
 * const result = await websign.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) });
 * expect(result.certificate.displayName).toBe("Test Holder");
 * await app.close();
 * ```
 *
 * @example
 * ```ts
 * // The person cancels:
 * const app = fakeApp({ failWith: { code: "UserCancelled" } });
 * await expect((await app.connect()).sign(options)).rejects.toMatchObject({ code: "UserCancelled" });
 * ```
 */
export function fakeApp(options: FakeAppOptions = {}): FakeApp {
  const directory = mkdtempSync(join(tmpdir(), "websign-fake-"));
  const log = join(directory, "requests.jsonl");
  const config = join(directory, "config.json");
  writeFileSync(log, "");
  writeFileSync(
    config,
    JSON.stringify({
      log,
      certificates: options.certificates ?? [SAMPLE_CERTIFICATE],
      failWith: options.failWith && { when: "choose", ...options.failWith },
      signature: options.signature,
      remembered: options.remembered ?? false,
      os: platformName(),
      arch: process.arch === "arm64" ? "aarch64" : "x86_64",
    }),
  );
  const connectOptions: ConnectOptions = {
    executable: process.execPath,
    executableArgs: [PROCESS, config],
  };
  const open: WebSign[] = [];
  return {
    options: connectOptions,
    async connect(extra = {}) {
      const websign = await WebSign.connect({ ...connectOptions, ...extra });
      open.push(websign);
      return websign;
    },
    requests: () =>
      readFileSync(log, "utf8")
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line) as Record<string, unknown>),
    async close() {
      await Promise.all(open.splice(0).map((websign) => websign.close()));
      rmSync(directory, { recursive: true, force: true });
    },
  };
}

function platformName(): "windows" | "macos" | "linux" {
  if (process.platform === "win32") return "windows";
  return process.platform === "darwin" ? "macos" : "linux";
}
