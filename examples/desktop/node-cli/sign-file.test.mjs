import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { promisify } from "node:util";

const run = promisify(execFile);
const script = new URL("./sign-file.mjs", import.meta.url).pathname.replace(/^\/(\w:)/, "$1");

test("signs the SHA-256 of the file against the fake app", async () => {
  const { stdout } = await run(process.execPath, [script, script]);
  const [summary, signatureHex] = stdout.trim().split("\n");
  assert.match(summary, /^signed by Test Holder with ECDSA$/);
  // The fake echoes the digest, so the "signature" is the file's SHA-256.
  const expected = createHash("sha256").update(readFileSync(script)).digest("hex");
  assert.equal(signatureHex, expected);
});

test("a missing argument is a usage error", async () => {
  await assert.rejects(run(process.execPath, [script]), (error) => error.code === 2);
});

test("a missing app explains itself", async () => {
  const env = { ...process.env, PATH: "", WEBSIGN_EXECUTABLE: "" };
  await assert.rejects(run(process.execPath, [script, script, "--app"], { env }), (error) => {
    assert.match(error.stderr, /^AppMissing: /);
    assert.match(error.stderr, /Install the WebeSign app/);
    return error.code === 1;
  });
});
