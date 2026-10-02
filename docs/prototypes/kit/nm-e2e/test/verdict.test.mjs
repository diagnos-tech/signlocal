import assert from "node:assert/strict";
import { test } from "node:test";

import { judge } from "../lib/verdict.mjs";

const DEV_ID = "nhnkdpljdgjflbflkhnkmfmcmodboeii";
const CLIENT_LINE = '2026-01-01T00:00:00.000Z pid=1 family=chromium ext=x event=client reason="installed"';

const healthy = () => ({
  done: true,
  failure: null,
  announce: { version: "0.1.0" },
  status: { state: "ready" },
  ping: { v: 1, ok: true, type: "pong", launch: { family: "chromium", extension_id: DEV_ID } },
  list: { ok: true, certificates: [], warnings: [] },
  wrongDigest: { ok: false, error: { code: "digest_length", message: "x" } },
  sign: null,
});

const judgeWith = (result, options = {}, log = [CLIENT_LINE]) =>
  judge(result, log, { devId: DEV_ID, expectSign: false, ...options });

test("a healthy run without certificates passes with a warning", () => {
  const verdict = judgeWith(healthy());
  assert.equal(verdict.ok, true);
  assert.match(verdict.warnings.join("\n"), /no certificates/);
});

test("a failing list is only a warning unless a signature is expected", () => {
  const result = healthy();
  result.list = { ok: false, error: { code: "internal", message: "no keystore" } };
  assert.equal(judgeWith(result).ok, true);
  assert.equal(judgeWith(result, { expectSign: true }).ok, false);
});

test("expecting a signature requires one that verified", () => {
  const result = healthy();
  result.list.certificates = [{ fingerprint: "a" }];
  assert.equal(judgeWith(result, { expectSign: true }).ok, false, "nothing signed");
  result.sign = { ok: true, verified: true };
  assert.equal(judgeWith(result, { expectSign: true }).ok, true);
  result.sign = { ok: true, verified: false };
  assert.equal(judgeWith(result, { expectSign: true }).ok, false, "a bad signature always fails");
  assert.equal(judgeWith(result).ok, false, "even when a signature was not required");
});

test("a failed signature is a warning unless expected", () => {
  const result = healthy();
  result.list.certificates = [{ fingerprint: "a" }];
  result.sign = { ok: false, error: { code: "pin_required", message: "set the PIN" } };
  assert.equal(judgeWith(result).ok, true);
  assert.equal(judgeWith(result, { expectSign: true }).ok, false);
});

test("each broken link in the chain is named", () => {
  const cases = [
    [(r) => { r.failure = "boom"; }, /boom/],
    [(r) => { r.announce = null; }, /announce/],
    [(r) => { r.status = { state: "not-installed" }; }, /not-installed/],
    [(r) => { r.ping = { ok: false, error: { code: "x", message: "y" } }; }, /ping failed/],
    [(r) => { r.ping.launch.family = "firefox"; }, /expected chromium/],
    [(r) => { r.ping.launch.extension_id = "z".repeat(32); }, /expected chromium/],
    [(r) => { r.wrongDigest = { ok: true }; }, /digest_length/],
  ];
  for (const [break_, pattern] of cases) {
    const result = healthy();
    break_(result);
    const verdict = judgeWith(result);
    assert.equal(verdict.ok, false);
    assert.match(verdict.reason, pattern);
  }
});

test("the host must have logged the extension's connection", () => {
  const verdict = judgeWith(healthy(), {}, ["unrelated line"]);
  assert.equal(verdict.ok, false);
  assert.match(verdict.reason, /connection record/);
});

test("no page result at all fails", () => {
  assert.equal(judgeWith(null).ok, false);
});
