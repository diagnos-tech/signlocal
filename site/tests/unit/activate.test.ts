import { describe, expect, test } from "bun:test";
import { activateState } from "../../src/client/activate.ts";

const status = (extension: boolean, ready: boolean) =>
  ({
    extension: { installed: extension },
    app: { installed: ready, outdated: false },
    remembered: false,
    ready,
  }) as never;

describe("activateState", () => {
  test("nothing shows until the button is pressed", () => {
    expect(activateState(false, status(true, true))).toBe("idle");
  });
  test("after pressing, it waits", () => {
    expect(activateState(true, undefined)).toBe("waiting");
    expect(activateState(true, status(true, false))).toBe("waiting");
  });
  test("ready means connected", () => expect(activateState(true, status(true, true))).toBe("done"));
  test("without the extension it says to add it first", () => {
    expect(activateState(true, status(false, false))).toBe("extension-first");
  });
});
