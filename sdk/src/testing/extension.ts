/**
 * The fake extension + app: answers the SDK's page messages on `window` the
 * way the content script, the extension and the app do together, with a
 * scripted "person" instead of the confirmation window.
 */

import { fromBase64, toBase64 } from "../convert.js";
import type { ErrorCode, ErrorDetails, PageReply, PageRequest } from "../generated/index.js";
import type { HashAlgorithm, SignatureAlgorithm } from "../types.js";
import { DIGEST_LENGTH } from "../validate.js";
import type { FakeCredential } from "./certificate.js";
import { chooseReply, errorReply, type FakeScenario, setupError, statusReply } from "./replies.js";
import type { Transport } from "./transport.js";
import type { CertificatePick, FakeRequest } from "./types.js";

interface Session {
  readonly record: { -readonly [K in keyof FakeRequest]: FakeRequest[K] };
  readonly hash: HashAlgorithm;
  readonly algorithms: readonly SignatureAlgorithm[] | undefined;
  credential: FakeCredential;
  algorithm: SignatureAlgorithm;
  seq: number;
}

export class FakeExtension {
  scenario: FakeScenario;
  remembered: boolean;
  pick: CertificatePick = 0;
  switchTo: CertificatePick | undefined;
  failure: { code: ErrorCode; details?: ErrorDetails } | undefined;
  readonly requests: FakeRequest[] = [];
  private readonly sessions = new Map<string, Session>();

  constructor(
    private readonly transport: Transport,
    readonly credentials: readonly FakeCredential[],
    options: { scenario: FakeScenario; remembered: boolean },
  ) {
    this.scenario = options.scenario;
    this.remembered = options.remembered;
  }

  /** One page frame (`discover` or `request`). */
  handle(frame: { kind?: unknown; id?: unknown; message?: unknown }): void {
    if (this.scenario === "extension-missing") return;
    if (frame.kind === "discover") this.transport.announce();
    if (frame.kind !== "request" || typeof frame.id !== "string") return;
    const request = frame.message as PageRequest;
    const id = frame.id;
    switch (request.type) {
      case "status":
        this.requests.push({ type: "status" });
        this.reply(id, statusReply(this.scenario, this.remembered));
        break;
      case "cancel": {
        const session = this.sessions.get(id);
        if (session) session.record.cancelled = true;
        this.sessions.delete(id);
        break;
      }
      case "sign.digest":
        void this.digest(id, request.seq, request.digest);
        break;
      default:
        this.begin(id, request);
    }
  }

  private begin(
    id: string,
    request: Extract<PageRequest, { type: "choose" | "sign.begin" }>,
  ): void {
    const algorithms = request.type === "choose" ? request.filter?.algorithms : request.algorithms;
    const preselected = request.type === "sign.begin" ? request.certificate : undefined;
    const record: Session["record"] =
      request.type === "choose"
        ? { type: "certificates", ...(algorithms && { algorithms }) }
        : {
            type: "sign",
            hash: request.hash,
            ...(algorithms && { algorithms }),
            ...(preselected && { certificate: preselected }),
          };
    this.requests.push(record);
    const failure = setupError(this.scenario) ?? this.takeFailure();
    if (failure) {
      this.reply(id, failure);
      return;
    }

    const usable = this.credentials.filter((c) =>
      c.wire.algorithms.some((a) => !algorithms || algorithms.includes(a)),
    );
    const chosen = preselected
      ? usable.find((c) => c.wire.fingerprint === preselected)
      : (this.find(this.pick, usable) ?? usable[0]);
    const now = Date.now() / 1000;
    if (!chosen) {
      this.reply(id, errorReply(preselected ? "CertificateUnavailable" : "NoCertificates"));
    } else if (request.type === "choose") {
      this.reply(id, chooseReply(chosen.wire));
    } else if (now < chosen.wire.notBefore || now > chosen.wire.notAfter) {
      this.reply(id, errorReply("CertificateNotValid"));
    } else {
      this.startSign(id, record, request.hash, algorithms, chosen);
    }
  }

  private startSign(
    id: string,
    record: Session["record"],
    hash: HashAlgorithm,
    algorithms: readonly SignatureAlgorithm[] | undefined,
    chosen: FakeCredential,
  ): void {
    const session: Session = {
      record,
      hash,
      algorithms,
      credential: chosen,
      algorithm: pickAlgorithm(chosen, algorithms),
      seq: 0,
    };
    this.sessions.set(id, session);
    this.needDigest(id, session);
  }

  private needDigest(id: string, session: Session): void {
    session.seq += 1;
    this.reply(id, {
      type: "sign.need_digest",
      seq: session.seq,
      certificate: session.credential.wire,
      hash: session.hash,
      algorithm: session.algorithm,
    });
  }

  private async digest(id: string, seq: number, digestText: string): Promise<void> {
    const session = this.sessions.get(id);
    if (!session || seq !== session.seq) return;
    const digest = fromBase64(digestText);
    if (digest.length !== DIGEST_LENGTH[session.hash]) {
      this.sessions.delete(id);
      this.reply(id, errorReply("InvalidRequest"));
      return;
    }
    const switched = this.switchTo === undefined ? undefined : this.find(this.switchTo);
    if (switched && switched !== session.credential && seq === 1) {
      // The person switches certificate after the first digest: the site prepares again.
      this.switchTo = undefined;
      session.credential = switched;
      session.algorithm = pickAlgorithm(switched, session.algorithms);
      this.needDigest(id, session);
      return;
    }
    const { credential, algorithm, hash } = session;
    const signature = await credential.sign(algorithm, hash, digest);
    if (this.sessions.get(id) !== session) return; // cancelled while signing
    this.sessions.delete(id);
    session.record.digest = digest;
    this.reply(id, {
      type: "sign.result",
      certificate: credential.wire,
      hash,
      algorithm,
      signature: toBase64(signature),
    });
  }

  private takeFailure(): PageReply | undefined {
    const failure = this.failure;
    this.failure = undefined;
    return failure && errorReply(failure.code, failure.details);
  }

  find(
    pick: CertificatePick,
    among: readonly FakeCredential[] = this.credentials,
  ): FakeCredential | undefined {
    const wanted =
      typeof pick === "number"
        ? this.credentials[pick]
        : this.credentials.find((c) => c.wire.fingerprint === pick);
    return wanted && among.includes(wanted) ? wanted : undefined;
  }

  private reply(id: string, message: PageReply): void {
    this.transport.reply(id, message);
  }
}

/** The first requested algorithm the key can produce; else the key's default. */
function pickAlgorithm(
  credential: FakeCredential,
  wanted: readonly SignatureAlgorithm[] | undefined,
): SignatureAlgorithm {
  const own = credential.wire.algorithms;
  return wanted?.find((a) => own.includes(a)) ?? own[0] ?? "ECDSA";
}
