import { describe, expect, it, vi } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { pageStatus, wireError } from "./helpers/fixtures";

useFakeEnvironment();

// status().problem names what sign() would reject with, so a site can show errorText(problem).
describe("status().problem", () => {
  it("ExtensionMissing when nobody announces", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const s = settle(env.sdk.status());
    await vi.advanceTimersByTimeAsync(1000);
    expect(expectValue(s.outcome()).problem).toBe("ExtensionMissing");
  });

  it("ClientOutdated when the extension only speaks newer protocols", async () => {
    const env = await loadSdk({ protocols: { min: 2, max: 3 } });
    const s = settle(env.sdk.status());
    await flush();
    const status = expectValue(s.outcome());
    expect(status).toMatchObject({ ready: false, problem: "ClientOutdated" });
    expect(status.extension.installed).toBe(true);
  });

  const replies: [string, Record<string, unknown>, string | undefined][] = [
    ["a healthy app", pageStatus(), undefined],
    ["no app in the status", pageStatus({ app: undefined }), "AppMissing"],
    ["an outdated app", pageStatus({ appOutdated: true }), "AppOutdated"],
    ["error AppMissing", wireError("AppMissing"), "AppMissing"],
    ["error AppOutdated", wireError("AppOutdated"), "AppOutdated"],
  ];
  for (const [name, reply, problem] of replies) {
    it(`${problem ?? "absent"} for ${name}, and ready agrees`, async () => {
      const env = await loadSdk();
      env.script.autoReply = () => [reply];
      const s = settle(env.sdk.status());
      await flush();
      const status = expectValue(s.outcome());
      expect(status.problem).toBe(problem);
      expect(status.ready).toBe(problem === undefined);
      expect("problem" in status).toBe(problem !== undefined);
    });
  }
});
