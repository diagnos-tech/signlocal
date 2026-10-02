/**
 * One Firefox tab driven over BiDi, shaped like the part of a Playwright
 * page the fixture calls use (`evaluate`), so `fixture.ts` serves both.
 */

import { writeFileSync } from "node:fs";

import type { Page } from "@playwright/test";

import type { Bidi } from "./bidi.ts";
import { fromTyped, type Typed, toTyped } from "./values.ts";

interface CallResult {
  readonly type: "success" | "exception";
  readonly result?: Typed;
  readonly exceptionDetails?: { readonly text?: string };
}

export class FirefoxTab {
  constructor(
    private readonly bidi: Bidi,
    private readonly context: string,
  ) {}

  /** Loads `url` and waits for its load event. */
  async goto(url: string): Promise<void> {
    await this.bidi.send("browsingContext.navigate", {
      context: this.context,
      url,
      wait: "complete",
    });
  }

  /**
   * Runs `fn(arg)` in the page and returns its (awaited) JSON result. The
   * function travels as source text, so it may use only its argument and
   * the page's globals, as with Playwright.
   */
  readonly evaluate = (async (fn: unknown, arg?: unknown) => {
    const call = await this.bidi.send<CallResult>("script.callFunction", {
      functionDeclaration: String(fn),
      awaitPromise: true,
      target: { context: this.context },
      arguments: [toTyped(arg)],
      resultOwnership: "none",
      serializationOptions: { maxObjectDepth: 20 },
    });
    if (call.type === "exception" || call.result === undefined) {
      throw new Error(`the page threw: ${call.exceptionDetails?.text ?? "no details"}`);
    }
    return fromTyped(call.result);
  }) as Page["evaluate"];

  /** Saves what the tab shows as a PNG. */
  async screenshot(path: string): Promise<void> {
    const shot = await this.bidi.send<{ data: string }>("browsingContext.captureScreenshot", {
      context: this.context,
    });
    writeFileSync(path, Buffer.from(shot.data, "base64"));
  }
}
