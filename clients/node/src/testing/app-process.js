#!/usr/bin/env node
// A well-behaved stand-in for `websign connect`, started by `fakeApp()`.
// Usage: node app-process.js <config.json> connect
//
// It is plain JavaScript (not TypeScript) so `node` can run it from src/ and
// from dist/ without a build step or a loader.
import { appendFileSync, readFileSync } from "node:fs";

const config = JSON.parse(readFileSync(process.argv[2], "utf8"));
const DIGEST_LENGTH = { "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 };
const APP = {
  version: "0.0.0-fake",
  protocols: { min: 1, max: 1 },
  os: config.os,
  arch: config.arch,
  channel: "direct",
};

function send(message, id) {
  const body = Buffer.from(JSON.stringify({ v: 1, id, ...message }));
  const header = Buffer.alloc(4);
  header.writeUInt32LE(body.length);
  process.stdout.write(Buffer.concat([header, body]));
}

const fail = (id, code, message) =>
  send({ type: "error", code, message: message ?? `the fake app answered ${code}` }, id);

/** The certificate a request may use, or undefined. */
function candidates(algorithms) {
  return config.certificates.filter(
    (c) => !algorithms || c.algorithms.some((a) => algorithms.includes(a)),
  );
}

/** One open signature, by request id. */
const signing = new Map();

function handle(request) {
  appendFileSync(config.log, `${JSON.stringify(request)}\n`);
  const { id, type } = request;
  const failing = config.failWith;
  if (type === "hello") return send({ type: "hello", protocol: 1, app: APP }, id);
  if (type === "status")
    return send({ type: "status", app: APP, remembered: config.remembered }, id);
  if (type === "diagnostics.open") return send({ type: "done" }, id);
  if (type === "cancel") {
    signing.delete(id);
    return fail(id, "Aborted", "cancelled by the caller");
  }
  if (type === "choose") {
    if (failing?.when === "choose") return fail(id, failing.code, failing.message);
    const found = candidates(request.filter?.algorithms);
    if (found.length === 0) return fail(id, "NoCertificates", "no certificate matches the filter");
    return send({ type: "choose.result", certificates: found }, id);
  }
  if (type === "sign.begin") return begin(request);
  if (type === "sign.digest") return finish(request);
  return fail(id, "InvalidRequest", `the fake app does not know ${type}`);
}

function begin(request) {
  const { id } = request;
  const failing = config.failWith;
  if (failing?.when === "choose") return fail(id, failing.code, failing.message);
  const found = candidates(request.algorithms).filter(
    (c) => request.certificate === undefined || c.fingerprint === request.certificate,
  );
  const certificate = found[0];
  if (!certificate)
    return fail(id, "CertificateUnavailable", "no such certificate in the fake app");
  const algorithm = certificate.algorithms.find(
    (a) => request.algorithms === undefined || request.algorithms.includes(a),
  );
  signing.set(id, { hash: request.hash, certificate, algorithm });
  send({ type: "sign.need_digest", seq: 1, hash: request.hash, algorithm, certificate }, id);
}

function finish(request) {
  const { id } = request;
  const open = signing.get(id);
  if (!open) return fail(id, "InvalidRequest", "no signature is open under this id");
  signing.delete(id);
  const digest = Buffer.from(request.digest, "base64");
  if (digest.length !== DIGEST_LENGTH[open.hash]) {
    return fail(id, "InvalidRequest", "the digest has the wrong length");
  }
  const failing = config.failWith;
  if (failing?.when === "confirm") return fail(id, failing.code, failing.message);
  send(
    {
      type: "sign.result",
      hash: open.hash,
      algorithm: open.algorithm,
      certificate: open.certificate,
      signature: config.signature ?? request.digest,
    },
    id,
  );
}

let buffered = Buffer.alloc(0);
process.stdin.on("data", (chunk) => {
  buffered = Buffer.concat([buffered, chunk]);
  while (buffered.length >= 4 && buffered.length >= 4 + buffered.readUInt32LE(0)) {
    const length = buffered.readUInt32LE(0);
    const request = JSON.parse(buffered.subarray(4, 4 + length).toString("utf8"));
    buffered = buffered.subarray(4 + length);
    handle(request);
  }
});
process.stdin.on("end", () => process.exit(0));
