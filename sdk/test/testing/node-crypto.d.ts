// The slice of node:crypto the tests use as an independent verifier. The SDK
// has no Node dependency, so it does not pull in @types/node for this.
declare module "node:crypto" {
  interface KeyObject {
    readonly type: string;
  }
  class X509Certificate {
    constructor(der: Uint8Array);
    readonly subject: string;
    readonly fingerprint256: string;
    readonly validTo: string;
    readonly publicKey: KeyObject;
    verify(key: KeyObject): boolean;
  }
  type VerifyKey =
    | KeyObject
    | { key: KeyObject; padding?: number; saltLength?: number; dsaEncoding?: string };
  function verify(
    algorithm: string,
    data: Uint8Array,
    key: VerifyKey,
    signature: Uint8Array,
  ): boolean;
  function createVerify(algorithm: string): {
    update(data: Uint8Array): { verify(key: VerifyKey, signature: Uint8Array): boolean };
  };
}
