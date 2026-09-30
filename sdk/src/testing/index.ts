/**
 * @websign/sdk/testing — a FAKE WebeSign extension and app for your unit and
 * end-to-end tests, and for demos without the real thing installed.
 *
 * Install it on `window` and the real `@websign/sdk` talks to it unchanged:
 * `status()`, `certificates()` and `sign()` work, `prepare` runs, and the
 * signatures are real ECDSA P-256 / RSA-2048 made with public test keys, so
 * your code can verify them. Scripted outcomes cover every state a visitor's
 * computer can be in.
 *
 * NEVER ship it: anyone can sign with the test keys. It refuses to install
 * outside localhost-like origins unless told otherwise, and warns on the
 * console every time it installs.
 *
 * @example
 * import { installFakeWebSign } from "@websign/sdk/testing";
 * import { sign } from "@websign/sdk";
 *
 * const fake = await installFakeWebSign();
 * const result = await sign({ hash: "SHA-256", prepare: (_, { hash }) => crypto.subtle.digest(hash, data) });
 * expect(await fake.verify(result, data)).toBe(true);
 * fake.uninstall();
 *
 * @packageDocumentation
 * @module @websign/sdk/testing
 */

import { toCertificate } from "../convert.js";
import type { ErrorCode, ErrorDetails } from "../generated/index.js";
import type { Bytes, Certificate, SignResult } from "../types.js";
import { createCredential, type FakeCertificateOptions } from "./certificate.js";
import { FakeExtension } from "./extension.js";
import { assertTestOrigin, warnInstalled, warnRealExtension } from "./guard.js";
import { announcement, type FakeScenario } from "./replies.js";
import { createTransport, type Transport } from "./transport.js";
import type { CertificatePick, FakeRequest } from "./types.js";
import { verifySignature } from "./verify.js";

export type { CertificatePick, FakeCertificateOptions, FakeRequest, FakeScenario };

/**
 * Options of {@link installFakeWebSign}; all optional.
 *
 * @example
 * await installFakeWebSign({ scenario: "ready", certificates: [{ key: "RSA" }], latencyMs: 300 });
 */
export interface FakeOptions {
  /** The state of the visitor's computer. Default `"ready"`. */
  readonly scenario?: FakeScenario;
  /** The person's certificates. Default: one EC P-256 and one RSA-2048. `[]` → `NoCertificates`. */
  readonly certificates?: readonly FakeCertificateOptions[];
  /** The person ticked "Remember this site" (`status().remembered`). Default false. */
  readonly remembered?: boolean;
  /** Delay of every answer, in ms, to see loading states. Default 0 (next microtask). */
  readonly latencyMs?: number;
  /** Install on a non-local origin (a staging server running e2e tests). Default false. */
  readonly allowAnyOrigin?: boolean;
}

/**
 * The installed fake: script what the person does next and inspect what
 * your page asked.
 */
export interface FakeWebSign {
  /** The certificates, as `sign()` and `certificates()` return them. */
  readonly certificates: readonly Certificate[];
  /** Every request your page made, oldest first. */
  readonly requests: readonly FakeRequest[];
  /**
   * Switches the state of the computer; the fake announces itself again, so
   * `onChange` listeners fire. A page never loses its extension, so
   * `"extension-missing"` is only accepted at install.
   *
   * @example
   * fake.setScenario("app-outdated");
   */
  setScenario(scenario: Exclude<FakeScenario, "extension-missing">): void;
  /**
   * The next `certificates()` or `sign()` fails with `code`, as if the app
   * or the person ended it. Status checks are not affected.
   *
   * @example
   * fake.failNext("UserCancelled");
   * await expect(sign(options)).rejects.toMatchObject({ code: "UserCancelled" });
   */
  failNext(code: ErrorCode, details?: ErrorDetails): void;
  /**
   * The certificate the person picks from now on (index or fingerprint).
   * Default: the first one the request can use.
   */
  choose(certificate: CertificatePick): void;
  /**
   * In the next `sign()`, the person switches to `certificate` after the
   * first digest, so `prepare` runs twice and the second certificate signs.
   *
   * @example
   * fake.switchDuringNextSign(1);
   */
  switchDuringNextSign(certificate: CertificatePick): void;
  /** Whether `result.signature` is a valid signature of `data` by `result.certificate`. */
  verify(result: SignResult, data: Bytes): Promise<boolean>;
  /** Stops answering. The SDK keeps its last view of the extension. */
  uninstall(): void;
}

const DEFAULT_CERTIFICATES: readonly FakeCertificateOptions[] = [{ key: "EC" }, { key: "RSA" }];

let installed: FakeWebSign | undefined;

/**
 * Installs the fake on the current `window` (replacing an earlier fake) and
 * announces it, as the real extension does when a page loads. Needs a DOM:
 * a browser, or a test environment such as jsdom or happy-dom.
 *
 * @example
 * // In a unit test (Vitest/Jest with a DOM environment):
 * let fake: FakeWebSign;
 * beforeEach(async () => (fake = await installFakeWebSign()));
 * afterEach(() => fake.uninstall());
 *
 * @example
 * // In your app, only in development: the import is dropped from production builds.
 * if (import.meta.env.DEV) await (await import("@websign/sdk/testing")).installFakeWebSign();
 *
 * @throws {Error} without a `window`, or on a non-local origin without `allowAnyOrigin`.
 */
export async function installFakeWebSign(options: FakeOptions = {}): Promise<FakeWebSign> {
  const win = (globalThis as { window?: Window }).window;
  if (win === undefined) {
    throw new Error(
      "installFakeWebSign() needs a window: run it in a browser, jsdom or happy-dom.",
    );
  }
  assertTestOrigin(win, options.allowAnyOrigin === true);
  installed?.uninstall();

  const specs = options.certificates ?? DEFAULT_CERTIFICATES;
  const credentials = await Promise.all(specs.map((spec, i) => createCredential(spec, i)));
  const scenario = options.scenario ?? "ready";
  let extension: FakeExtension | undefined;
  const transport: Transport = createTransport({
    window: win,
    latencyMs: options.latencyMs ?? 0,
    announcement: () => announcement(extension?.scenario ?? scenario),
    onFrame: (frame) => extension?.handle(frame),
    onForeignExtension: warnRealExtension,
  });
  extension = new FakeExtension(transport, credentials, {
    scenario,
    remembered: options.remembered ?? false,
  });
  const engine = extension;
  warnInstalled();
  if (scenario !== "extension-missing") transport.announce();

  const fake: FakeWebSign = {
    certificates: credentials.map((c) => toCertificate(c.wire)),
    requests: engine.requests,
    setScenario(next) {
      if ((next as FakeScenario) === "extension-missing") {
        throw new Error(
          'A page never loses its extension: install the fake with { scenario: "extension-missing" } instead.',
        );
      }
      engine.scenario = next;
      transport.announce();
    },
    failNext: (code, details) => {
      engine.failure = { code, ...(details && { details }) };
    },
    choose: (pick) => {
      engine.pick = pick;
    },
    switchDuringNextSign: (pick) => {
      engine.switchTo = pick;
    },
    verify: (result, data) => verifySignature(credentials, result, data),
    uninstall: () => {
      transport.stop();
      if (installed === fake) installed = undefined;
    },
  };
  installed = fake;
  return fake;
}
