/** One connection to the app. */

import { decodeCertificate } from "./certificate.js";
import { aborted, SignlocalError } from "./errors.js";
import type { DiagnosticsTab, StatusReply } from "./generated/index.js";
import { findExecutable } from "./locate.js";
import { DEFAULT_TIMINGS, Session } from "./session.js";
import { runSign } from "./sign.js";
import type {
  Certificate,
  CertificateOptions,
  ConnectOptions,
  SignOptions,
  SignResult,
} from "./types.js";
import { checkAlgorithms } from "./validate.js";
import { CLIENT_NAME, VERSION } from "./version.js";

/**
 * A running `signlocal connect` child. The app lives only as long as this
 * object: call {@link Signlocal.close} when done, or it exits when your process
 * does. One connection serves one caller; open several for parallel work.
 *
 * @example
 * ```ts
 * const signlocal = await Signlocal.connect();
 * try {
 *   const { signature } = await signlocal.sign({
 *     hash: "SHA-256",
 *     prepare: () => sha256OfMyDocument(),
 *   });
 * } finally {
 *   await signlocal.close();
 * }
 * ```
 */
export class Signlocal {
  private constructor(private readonly session: Session) {}

  /**
   * Starts the app and negotiates the protocol. Rejects with `AppMissing`
   * when the app is not installed or exits at once, and with `ClientOutdated`
   * or `AppOutdated` when no protocol version is common.
   *
   * @example
   * ```ts
   * import { Signlocal, SignlocalError } from "@signlocal/desktop";
   *
   * try {
   *   const signlocal = await Signlocal.connect({ clientName: "my-invoicing-app" });
   *   await signlocal.close();
   * } catch (error) {
   *   if (error instanceof SignlocalError && error.code === "AppMissing") {
   *     console.error(error.hint); // tells the person how to install it
   *   }
   * }
   * ```
   */
  static async connect(options: ConnectOptions = {}): Promise<Signlocal> {
    const executable = options.executable ?? findExecutable();
    if (executable === undefined) {
      throw new SignlocalError("AppMissing", "the signlocal executable was not found");
    }
    const session = await Session.open(
      executable,
      {
        name: options.clientName ?? CLIENT_NAME,
        version: options.clientVersion ?? VERSION,
      },
      DEFAULT_TIMINGS,
      options.executableArgs,
    );
    return new Signlocal(session);
  }

  /**
   * Asks the app who it is (`app.version`, OS, channel) and whether this
   * program is remembered. Never opens a window; `remembered` says whether
   * {@link Signlocal.certificates} will skip it too.
   *
   * @example
   * ```ts
   * const { app, remembered } = await signlocal.status();
   * console.log(`signlocal ${app.version}`, remembered ? "(remembered)" : "");
   * ```
   */
  async status(): Promise<StatusReply> {
    const reply = await this.session.request("status").result;
    if (reply.type !== "status") throw unexpected(reply.type, "status");
    return { app: reply.app, remembered: reply.remembered };
  }

  /**
   * `choose`: the person picks a certificate (or a remembered program gets
   * the ones it used before). Never the machine's whole list. `der` and
   * `chain` are decoded to bytes. Same options as `@signlocal/sdk`.
   *
   * @throws {@link SignlocalError} `NoCertificates` when the person chose none,
   * `UserCancelled`, `Aborted` (signal), `Timeout`.
   *
   * @example
   * ```ts
   * const [certificate] = await signlocal.certificates({ algorithm: "ECDSA" });
   * console.log(certificate.displayName, certificate.fingerprint);
   * // Later: signlocal.sign({ certificate, ... }) skips the choice.
   * ```
   */
  async certificates(options: CertificateOptions = {}): Promise<Certificate[]> {
    const algorithms = checkAlgorithms(options.algorithm);
    const { signal } = options;
    if (signal?.aborted) throw aborted(signal.reason);
    const exchange = this.session.request("choose", algorithms ? { filter: { algorithms } } : {});
    const onAbort = () => exchange.fail(aborted(signal?.reason));
    signal?.addEventListener("abort", onAbort, { once: true });
    try {
      const reply = await exchange.result;
      if (reply.type !== "choose.result" || !Array.isArray(reply.certificates)) {
        throw unexpected(reply.type, "choose");
      }
      // The app reports an empty choice as NoCertificates; an empty list would be a bug there.
      if (reply.certificates.length === 0) {
        throw new SignlocalError("NoCertificates", "no certificate was chosen");
      }
      return reply.certificates.map(decodeCertificate);
    } finally {
      signal?.removeEventListener("abort", onAbort);
    }
  }

  /**
   * `sign.begin` … `sign.result`. The app shows this program's name in its
   * window, the person picks a certificate, `options.prepare` supplies the
   * digest for it, and the person confirms with the PIN.
   *
   * @throws {@link SignlocalError} `UserCancelled`, `PinLocked`, `TokenRemoved`,
   * `Timeout`, `Aborted` (signal or throwing `prepare`), `InvalidRequest`
   * (wrong digest length, or an app asking for another hash/algorithm), …
   *
   * @example
   * ```ts
   * import { createHash } from "node:crypto";
   *
   * const { signature, certificate, algorithm } = await signlocal.sign({
   *   hash: "SHA-256",
   *   algorithm: ["ECDSA", "RSASSA-PSS"],
   *   signal: AbortSignal.timeout(120_000),
   *   prepare: (certificate, { algorithm }) =>
   *     createHash("sha256").update(documentFor(certificate, algorithm)).digest(),
   * });
   * ```
   */
  sign(options: SignOptions): Promise<SignResult> {
    return runSign(this.session, options);
  }

  /**
   * Opens the app's diagnostics window (in its own process), for example to
   * help a person whose token is not listed.
   *
   * @example
   * ```ts
   * await signlocal.openDiagnostics("devices"); // or "browsers", "certificates", "help"
   * ```
   */
  async openDiagnostics(tab?: DiagnosticsTab): Promise<void> {
    const reply = await this.session.request("diagnostics.open", tab ? { tab } : {}).result;
    if (reply.type !== "done") throw unexpected(reply.type, "diagnostics.open");
  }

  /**
   * Closes stdin; the app exits (killed after 5 s if it does not). Idempotent.
   * Pending calls reject.
   *
   * @example
   * ```ts
   * await signlocal.close();
   * ```
   */
  close(): Promise<void> {
    return this.session.close();
  }
}

function unexpected(got: string, request: string): SignlocalError {
  return new SignlocalError("Internal", `unexpected reply ${got} to ${request}`);
}
