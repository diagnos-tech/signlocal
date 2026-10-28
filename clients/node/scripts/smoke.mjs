// Run after `build`: the package must import as ESM by name, like a consumer.
import assert from "node:assert/strict";
import { Signlocal, SignlocalError } from "@signlocal/desktop";
import { fakeApp } from "@signlocal/desktop/testing";

const app = fakeApp();
const signlocal = await app.connect();
const { signature } = await signlocal.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) });
assert.equal(signature.byteLength, 32);
await app.close();
assert.equal(typeof Signlocal.connect, "function");
assert.ok(new SignlocalError("Busy", "x").hint.length > 0);
console.log("esm ok");
