#!/usr/bin/env node
// Signs the SHA-256 of a file with the person's certificate.
//
//   node sign-file.mjs <file>         uses the fake app: runs anywhere, e.g. in CI
//   node sign-file.mjs <file> --app   uses the installed SignLocal app
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { WebSign, WebSignError } from "@websign/desktop";

const [path, flag] = process.argv.slice(2);
if (!path || (flag !== undefined && flag !== "--app")) {
  console.error("usage: sign-file.mjs <file> [--app]");
  process.exit(2);
}

// The fake is a dev tool: load it only when it is used.
const fake = flag === "--app" ? undefined : (await import("@websign/desktop/testing")).fakeApp();
let websign;
try {
  const document = await readFile(path);
  websign = await (fake ? fake.connect() : WebSign.connect({ clientName: "websign-sign-file" }));
  const { signature, certificate, algorithm } = await websign.sign({
    hash: "SHA-256",
    signal: AbortSignal.timeout(120_000),
    // The app calls this once the person picked a certificate.
    prepare: () => createHash("sha256").update(document).digest(),
  });
  console.log(`signed by ${certificate.displayName} with ${algorithm}`);
  console.log(Buffer.from(signature).toString("hex"));
} catch (error) {
  if (!(error instanceof WebSignError)) throw error;
  console.error(`${error.code}: ${error.message}\n${error.hint}\n${error.docsUrl}`);
  process.exitCode = error.code === "UserCancelled" ? 3 : 1;
} finally {
  await websign?.close();
  await fake?.close();
}
