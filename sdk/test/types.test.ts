import { describe, expectTypeOf, it } from "vitest";
import type { ErrorCode, PrepareContext, SignResult, Status, StatusProblem } from "../src/index";
import { isWebSignError, sign } from "../src/index";

// Compile-time contract of the public types: `bun run typecheck` fails when they regress.
// Nothing here runs a signature: the calls are only type-checked.
describe("type-level ergonomics", () => {
  it("prepare's context and the result follow hash and algorithm literals", () => {
    const typed = () =>
      sign({
        hash: "SHA-384",
        algorithm: "ECDSA",
        prepare: (_certificate, context) => {
          expectTypeOf(context).toEqualTypeOf<PrepareContext<"SHA-384", "ECDSA">>();
          return new Uint8Array(48);
        },
      });
    expectTypeOf(typed).returns.resolves.toEqualTypeOf<SignResult<"SHA-384", "ECDSA">>();
  });

  it("a list of algorithms narrows to that union; none means all of them", () => {
    const listed = () =>
      sign({
        hash: "SHA-256",
        algorithm: ["ECDSA", "RSASSA-PSS"],
        prepare: () => new Uint8Array(32),
      });
    expectTypeOf(listed)
      .returns.resolves.toHaveProperty("algorithm")
      .toEqualTypeOf<"ECDSA" | "RSASSA-PSS">();
    const any = () => sign({ hash: "SHA-512", prepare: async () => new ArrayBuffer(64) });
    expectTypeOf(any)
      .returns.resolves.toHaveProperty("algorithm")
      .toEqualTypeOf<"ECDSA" | "RSASSA-PKCS1-v1_5" | "RSASSA-PSS">();
  });

  it("rejects unknown hashes, algorithms and non-byte digests at compile time", () => {
    // @ts-expect-error SHA-1 is not offered
    void (() => sign({ hash: "SHA-1", prepare: () => new Uint8Array(20) }));
    // @ts-expect-error EdDSA signs messages, not hashes
    void (() => sign({ hash: "SHA-256", algorithm: "EdDSA", prepare: () => new Uint8Array(32) }));
    // @ts-expect-error a hex string is not bytes
    void (() => sign({ hash: "SHA-256", prepare: () => "ab".repeat(32) }));
  });

  it("isWebSignError narrows the code", () => {
    const error: unknown = undefined;
    if (isWebSignError(error, "UserCancelled", "Aborted")) {
      expectTypeOf(error.code).toEqualTypeOf<"UserCancelled" | "Aborted">();
    }
    if (isWebSignError(error)) expectTypeOf(error.code).toEqualTypeOf<ErrorCode>();
  });

  it("status().problem is an error code and dates are Dates", () => {
    expectTypeOf<Status["problem"]>().toEqualTypeOf<StatusProblem | undefined>();
    expectTypeOf<StatusProblem>().toExtend<ErrorCode>();
    expectTypeOf<SignResult["certificate"]["notAfter"]>().toEqualTypeOf<Date>();
    expectTypeOf<SignResult["signature"]>().toEqualTypeOf<Uint8Array<ArrayBuffer>>();
  });

  it("bytes the SDK returns go straight into WebCrypto", () => {
    const hashDer = (result: SignResult) => crypto.subtle.digest("SHA-256", result.certificate.der);
    const hashChain = (result: SignResult) =>
      result.certificate.chain.map((der) => crypto.subtle.digest("SHA-256", der));
    expectTypeOf(hashDer).returns.resolves.toEqualTypeOf<ArrayBuffer>();
    expectTypeOf(hashChain).returns.toEqualTypeOf<Promise<ArrayBuffer>[]>();
  });
});
