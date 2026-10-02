import { expect } from "vitest";
import type { Env, Outcome } from "./env";

/** Asserts a settled outcome is a `WebSignError` with `code`; returns the error. */
export function expectError(env: Env, outcome: Outcome<unknown> | undefined, code: string) {
  expect(outcome, "promise still pending").toBeDefined();
  if (!outcome || outcome.ok)
    throw new Error(`expected rejection ${code}, got ${JSON.stringify(outcome)}`);
  expect(outcome.error).toBeInstanceOf(env.sdk.WebSignError);
  const error = outcome.error as InstanceType<Env["sdk"]["WebSignError"]>;
  expect(error.code).toBe(code);
  expect(error.name).toBe("WebSignError");
  return error;
}

export function expectValue<T>(outcome: Outcome<T> | undefined): T {
  expect(outcome, "promise still pending").toBeDefined();
  if (!outcome?.ok)
    throw new Error(
      `expected resolution, got ${String(outcome && (outcome as { error: unknown }).error)}`,
    );
  return outcome.value;
}
