import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { promisify } from "node:util";

import { expect, test } from "@playwright/test";

import { WebSign } from "../../clients/node/dist/index.js";
import { appEnv } from "../lib/app.ts";
import { newMessage } from "../lib/fixture.ts";
import { env, keyOf } from "../lib/suite.ts";
import { verifies } from "../lib/verify.ts";

const run = promisify(execFile);

// Scenario 7 (docs/architecture/testing.md §5): programs on this computer
// sign with the same keys and window, through `websign connect`.
test.describe("desktop callers", () => {
  test("@websign/desktop signs through websign connect", async () => {
    const key = keyOf("EC");
    const message = Buffer.from(newMessage(), "hex");
    // The client starts `websign connect` with this process's environment.
    const saved = { ...process.env };
    Object.assign(process.env, appEnv(env, "sign"));
    const websign = await WebSign.connect({ executable: env.app, clientName: "websign-e2e" });
    try {
      const status = await websign.status();
      expect(status.app.version).toBeTruthy();
      const prepared: string[] = [];
      const result = await websign.sign({
        hash: "SHA-384",
        certificate: key.fingerprint,
        prepare: (certificate) => {
          prepared.push(certificate.fingerprint);
          return createHash("sha384").update(message).digest();
        },
      });
      expect(prepared).toEqual([key.fingerprint]);
      expect(result.certificate.fingerprint).toBe(key.fingerprint);
      expect(
        verifies(key.certificate, "SHA-384", result.algorithm, message, result.signature),
      ).toBe(true);
    } finally {
      await websign.close();
      process.env = saved;
    }
  });

  test("websign sign signs a digest from the command line", async () => {
    const key = keyOf("RSA");
    const message = Buffer.from(newMessage(), "hex");
    const digest = createHash("sha256").update(message).digest("hex");
    const { stdout } = await run(
      env.app,
      ["sign", "--hash", "SHA-256", "--digest", digest, "--certificate", key.fingerprint],
      { env: appEnv(env, "sign"), timeout: 90_000 },
    );
    const reply = JSON.parse(stdout) as {
      type: string;
      algorithm: "RSASSA-PKCS1-v1_5";
      signature: string;
    };
    expect(reply.type).toBe("sign.result");
    const signature = Buffer.from(reply.signature, "base64");
    expect(verifies(key.certificate, "SHA-256", reply.algorithm, message, signature)).toBe(true);
  });
});
