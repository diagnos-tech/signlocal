import { vi } from "vitest";
import type { SignOptions } from "../../src/types";
import { type Env, settle } from "./env";
import { flush } from "./fake-script";
import { bytes } from "./fixtures";

/** Starts a sign() against an announced extension and waits for sign.begin. */
export async function beginSign(env: Env, overrides: Partial<SignOptions> = {}) {
  const prepare = vi.fn(
    (..._args: unknown[]): Uint8Array | ArrayBuffer | Promise<Uint8Array | ArrayBuffer> =>
      bytes(32),
  );
  const options = { hash: "SHA-256", prepare, ...overrides } as SignOptions;
  const s = settle(env.sdk.sign(options));
  await flush();
  const id = env.script.only("sign.begin").id;
  return { s, prepare, id };
}

/** Digest requests posted so far, as `{seq, digest}` messages. */
export function digestsSent(env: Env): { seq: number; digest: string }[] {
  return env.script.requestsOfType("sign.digest").map((r) => ({
    seq: r.message.seq as number,
    digest: r.message.digest as string,
  }));
}

/** A promise a test settles by hand, to model a slow `prepare`. */
export function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}
