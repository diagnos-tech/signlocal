/** One connection to the app. */

import { decodeCertificate } from "./certificate.js";
import { aborted, WebSignError } from "./errors.js";
import type { DiagnosticsTab, StatusReply } from "./generated/index.js";
import { findExecutable } from "./locate.js";
import { Session } from "./session.js";
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
 * A running `websign connect` child. The app lives only as long as this
 * object: call {@link WebSign.close} when done, or it exits when your process
 * does. One connection serves one caller; open several for parallel work.
 */
export class WebSign {
  private constructor(private readonly session: Session) {}

  /**
   * Starts the app and negotiates the protocol. Rejects with `AppMissing`
   * when the app is not installed or exits at once, and with `ClientOutdated`
   * or `AppOutdated` when no protocol version is common.
   */
  static async connect(options: ConnectOptions = {}): Promise<WebSign> {
    const executable = options.executable ?? findExecutable();
    if (executable === undefined) {
      throw new WebSignError("AppMissing", "the websign executable was not found");
    }
    const session = await Session.open(executable, {
      name: options.clientName ?? CLIENT_NAME,
      version: options.clientVersion ?? VERSION,
    });
    return new WebSign(session);
  }

  /** `status`: never opens a window; `remembered` says whether `certificates()` will too. */
  async status(): Promise<StatusReply> {
    const reply = await this.session.request("status").result;
    if (reply.type !== "status") throw unexpected(reply.type, "status");
    return { app: reply.app, remembered: reply.remembered };
  }

  /**
   * `choose`: the person picks a certificate (or a remembered program gets
   * the ones it used before). Never the machine's whole list. `der` and
   * `chain` are decoded to bytes. Same options as `@websign/sdk`.
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
        throw new WebSignError("NoCertificates", "no certificate was chosen");
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
   */
  sign(options: SignOptions): Promise<SignResult> {
    return runSign(this.session, options);
  }

  /** `diagnostics.open`: opens the diagnostics window in its own process. */
  async openDiagnostics(tab?: DiagnosticsTab): Promise<void> {
    const reply = await this.session.request("diagnostics.open", tab ? { tab } : {}).result;
    if (reply.type !== "done") throw unexpected(reply.type, "diagnostics.open");
  }

  /** Closes stdin; the app exits (killed after 5 s if it does not). Idempotent. */
  close(): Promise<void> {
    return this.session.close();
  }
}

function unexpected(got: string, request: string): WebSignError {
  return new WebSignError("Internal", `unexpected reply ${got} to ${request}`);
}
