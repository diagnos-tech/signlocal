#!/usr/bin/env node
// A stand-in for websign-probe that speaks the same native messaging
// protocol with one canned certificate, so the test harness itself (page,
// extension, verdict, `--expect-sign`) can be exercised without any keystore:
//
//   npm run selftest        (Linux and macOS: the manifest points at this script)
//
// It implements only what run.mjs needs: `register --user-data-dir` and the
// host loop for ping, list and sign.

import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import { endianness, tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { readProject } from "../../extension/project-config.mjs";

const project = readProject();
const LITTLE = endianness() === "LE";
const FINGERPRINT = "ab".repeat(32);
const DIGEST_BYTES = { "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 };

function register(args) {
  const dir = args[args.indexOf("--user-data-dir") + 1];
  const hosts = join(dir, "NativeMessagingHosts");
  mkdirSync(hosts, { recursive: true });
  writeFileSync(
    join(hosts, `${project.nativeHost}.json`),
    JSON.stringify({
      name: project.nativeHost,
      description: "fake host for the harness self-test",
      path: fileURLToPath(import.meta.url),
      type: "stdio",
      allowed_origins: [`chrome-extension://${project.devId}/`],
    }),
  );
}

function log(line) {
  const path = join(tmpdir(), `${project.slug}-probe-host.log`);
  appendFileSync(path, `${new Date().toISOString()} pid=${process.pid} family=chromium ext=${project.devId} ${line}\n`);
}

function reply(request, fields) {
  const body = Buffer.from(JSON.stringify({ v: 1, id: request.id, ...fields }));
  const header = Buffer.alloc(4);
  LITTLE ? header.writeUInt32LE(body.length) : header.writeUInt32BE(body.length);
  process.stdout.write(Buffer.concat([header, body]));
}

function handle(request) {
  if (request.v !== 1) return reply(request, { ok: false, error: { code: "unsupported_version", message: "v" } });
  switch (request.type) {
    case "ping":
      if (request.client) log(`event=client reason="${request.client.reason}"`);
      return reply(request, {
        ok: true,
        type: "pong",
        app: { name: "fake-probe", version: "0.1.0" },
        os: process.platform,
        arch: process.arch,
        launch: { family: "chromium", extension_id: project.devId },
      });
    case "list":
      return reply(request, {
        ok: true,
        type: "certificates",
        warnings: [],
        certificates: [
          {
            fingerprint: FINGERPRINT,
            display_name: "Fake Holder",
            can_sign: true,
            algorithms: ["ECDSA"],
            key: "EC P-256",
          },
        ],
      });
    case "sign": {
      const expected = DIGEST_BYTES[request.hash];
      const actual = Buffer.from(String(request.digest), "base64").length;
      if (!expected || actual !== expected) {
        return reply(request, { ok: false, error: { code: "digest_length", message: `expected ${expected}, got ${actual}` } });
      }
      if (request.fingerprint !== FINGERPRINT) {
        return reply(request, { ok: false, error: { code: "not_found", message: "no such certificate" } });
      }
      return reply(request, {
        ok: true,
        type: "signature",
        signature: Buffer.alloc(64, 7).toString("base64"),
        api: "fake",
        elapsed_ms: 1,
        verified: process.env.FAKE_UNVERIFIED !== "1",
      });
    }
    default:
      return reply(request, { ok: false, error: { code: "unknown_type", message: "type" } });
  }
}

function serve() {
  let buffer = Buffer.alloc(0);
  process.stdin.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    while (buffer.length >= 4) {
      const length = LITTLE ? buffer.readUInt32LE(0) : buffer.readUInt32BE(0);
      if (buffer.length < 4 + length) return;
      const request = JSON.parse(buffer.subarray(4, 4 + length).toString("utf8"));
      buffer = buffer.subarray(4 + length);
      handle(request);
    }
  });
}

const [command, ...rest] = process.argv.slice(2);
if (command === "register") register(rest);
else serve();
