#!/usr/bin/env node
// Signs the SHA-256 of a file with the person's certificate.
//
//   node sign-file.mjs <file>         uses the fake app: runs anywhere, e.g. in CI
//   node sign-file.mjs <file> --app   uses the installed signlocal app
// #region site:node
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { Signlocal, SignlocalError } from "@signlocal/desktop";

const [path, flag] = process.argv.slice(2);
if (!path || (flag !== undefined && flag !== "--app")) {
  console.error("usage: sign-file.mjs <file> [--app]");
  process.exit(2);
}

// The fake is a dev tool: load it only when it is used.
const fake = flag === "--app" ? undefined : (await import("@signlocal/desktop/testing")).fakeApp();
let signlocal;
try {
  const document = await readFile(path);
  signlocal = await (fake
    ? fake.connect()
    : Signlocal.connect({ clientName: "signlocal-sign-file" }));
  const { signature, certificate, algorithm } = await signlocal.sign({
    hash: "SHA-256",
    signal: AbortSignal.timeout(120_000),
    // The app calls this once the person picked a certificate.
    prepare: () => createHash("sha256").update(document).digest(),
  });
  console.log(`signed by ${certificate.displayName} with ${algorithm}`);
  console.log(Buffer.from(signature).toString("hex"));
} catch (error) {
  if (!(error instanceof SignlocalError)) throw error;
  console.error(`${error.code}: ${error.message}\n${error.hint}\n${error.docsUrl}`);
  process.exitCode = error.code === "UserCancelled" ? 3 : 1;
} finally {
  await signlocal?.close();
  await fake?.close();
}
// #endregion site:node
