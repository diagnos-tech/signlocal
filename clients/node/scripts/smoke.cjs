// Node >= 20.19 loads the ESM-only package through require().
const assert = require("node:assert/strict");
const { WebSign, WebSignError } = require("@websign/desktop");
const { fakeApp } = require("@websign/desktop/testing");

(async () => {
  const app = fakeApp();
  const websign = await app.connect();
  const { signature } = await websign.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) });
  assert.equal(signature.byteLength, 32);
  await app.close();
  assert.equal(typeof WebSign.connect, "function");
  assert.ok(new WebSignError("Busy", "x").hint.length > 0);
  console.log("cjs ok");
})();
