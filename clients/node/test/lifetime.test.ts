import { describe, expect, it } from "vitest";
import { WebSign } from "../src/index";
import { APP, fake, helloRule, send } from "./support";

/** Handles that keep Node's event loop alive (unref'd ones are not listed). */
const holding = () =>
  process.getActiveResourcesInfo().filter((name) => name === "ProcessWrap" || name === "PipeWrap")
    .length;

describe("event loop", () => {
  it("does not keep the caller alive while idle, but does while a request is open", async () => {
    const before = holding();
    const app = fake({
      rules: [
        helloRule(),
        {
          on: "status",
          do: [{ delay: 100 }, send({ type: "status", app: APP, remembered: false })],
        },
      ],
    });
    const websign = await WebSign.connect({ executable: app.executable });
    expect(holding()).toBe(before);
    const status = websign.status();
    expect(holding()).toBeGreaterThan(before);
    await status;
    expect(holding()).toBe(before);
    await websign.close();
  });
});
