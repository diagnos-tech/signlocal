// Node >= 20.19 loads the ESM-only package through require().
const assert = require("node:assert/strict");
const { Signlocal, SignlocalError } = require("@signlocal/desktop");
const { fakeApp } = require("@signlocal/desktop/testing");

(async () => {
  const app = fakeApp();
  const signlocal = await app.connect();
  const { signature } = await signlocal.sign({
    hash: "SHA-256",
    prepare: () => new Uint8Array(32),
  });
  assert.equal(signature.byteLength, 32);
  await app.close();
  assert.equal(typeof Signlocal.connect, "function");
  assert.ok(new SignlocalError("Busy", "x").hint.length > 0);
  console.log("cjs ok");
})();
