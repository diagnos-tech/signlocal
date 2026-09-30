/**
 * Typed calls into the fixture page (`fixtures/page.html`), which runs the
 * SDK exactly as a site would and answers in plain JSON.
 */

import { randomBytes } from "node:crypto";

import type { Page } from "@playwright/test";

import type { HashName, SignatureAlgorithm } from "./verify.ts";

/** What `status()` reported. */
export interface StatusReply {
  readonly extension: { readonly installed: boolean };
  readonly app: { readonly installed: boolean; readonly outdated: boolean };
  readonly remembered: boolean;
  readonly ready: boolean;
}

/** A certificate `prepare` was called with. */
export interface Prepared {
  readonly fingerprint: string;
  readonly algorithm: SignatureAlgorithm;
}

export type SignReply =
  | {
      readonly ok: true;
      readonly prepared: readonly Prepared[];
      readonly hash: HashName;
      readonly algorithm: SignatureAlgorithm;
      readonly signature: string;
      readonly certificate: string;
      readonly fingerprint: string;
    }
  | { readonly ok: false; readonly code: string; readonly prepared: readonly Prepared[] };

export type CertificatesReply =
  | { readonly ok: true; readonly fingerprint: string; readonly der: string }
  | { readonly ok: false; readonly code: string };

/** What the page signs: the hash of `message`. */
export interface SignRequest {
  readonly hash: HashName;
  readonly message: string;
  readonly algorithm?: SignatureAlgorithm;
  readonly certificate?: string;
  readonly digestLength?: number;
  readonly holdPrepare?: boolean;
}

/** A random message, hex. */
export function newMessage(): string {
  return randomBytes(48).toString("hex");
}

/** Waits for the page's script; the SDK itself waits for the extension. */
export async function ready(page: Page): Promise<void> {
  await page.locator("#result", { hasText: "ready" }).waitFor();
}

export function status(page: Page): Promise<StatusReply> {
  return page.evaluate(() => (window as unknown as E2eWindow).websignE2e.status());
}

export function sign(page: Page, request: SignRequest): Promise<SignReply> {
  return page.evaluate((r) => (window as unknown as E2eWindow).websignE2e.sign(r), request);
}

export function certificates(page: Page): Promise<CertificatesReply> {
  return page.evaluate(() => (window as unknown as E2eWindow).websignE2e.certificates());
}

/** Starts a signature without waiting for it. */
export async function start(page: Page, request: SignRequest): Promise<void> {
  await page.evaluate((r) => (window as unknown as E2eWindow).websignE2e.start(r), request);
}

/** The reply of the signature `start` began. */
export function result(page: Page): Promise<SignReply> {
  return page.evaluate(() => (window as unknown as E2eWindow).websignE2e.result());
}

/**
 * Waits until the page's `prepare` was called: the app asked for the
 * digest. Polls on a timer: a background tab draws no animation frames.
 */
export async function prepareCalled(page: Page): Promise<void> {
  await page.waitForFunction(
    () => (window as unknown as E2eWindow).websignE2e.preparedCount() > 0,
    undefined,
    { polling: 100, timeout: 60_000 },
  );
}

/** Withdraws the started signature through its AbortSignal. */
export async function abort(page: Page): Promise<void> {
  await page.evaluate(() => (window as unknown as E2eWindow).websignE2e.abort());
}

interface E2eWindow {
  readonly websignE2e: {
    status(): Promise<StatusReply>;
    sign(request: SignRequest): Promise<SignReply>;
    certificates(): Promise<CertificatesReply>;
    start(request: SignRequest): boolean;
    result(): Promise<SignReply>;
    preparedCount(): number;
    abort(): boolean;
  };
}
