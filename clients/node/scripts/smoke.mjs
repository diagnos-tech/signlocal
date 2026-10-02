// Run after `build`: the package must import as ESM by name, like a consumer.
import assert from "node:assert/strict";
import { WebSign, WebSignError } from "@websign/desktop";
import { fakeApp } from "@websign/desktop/testing";

const app = fakeApp();
const websign = await app.connect();
const { signature } = await websign.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) });
assert.equal(signature.byteLength, 32);
await app.close();
assert.equal(typeof WebSign.connect, "function");
assert.ok(new WebSignError("Busy", "x").hint.length > 0);
console.log("esm ok");
