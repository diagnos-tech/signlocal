import { describe, expect, it } from "vitest";
import { WebSign, WebSignError } from "../src/index";
import { Session } from "../src/session";
import { VERSION } from "../src/version";
import { eventually, fake, helloRule, isAlive, send } from "./support";

const rejection = async (promise: Promise<unknown>): Promise<WebSignError> => {
  try {
    await promise;
  } catch (error) {
    expect(error).toBeInstanceOf(WebSignError);
    return error as WebSignError;
  }
  throw new Error("expected a rejection");
};

describe("WebSign.connect", () => {
  it("spawns `connect` and sends hello with identity and protocol range", async () => {
    const app = fake({ rules: [helloRule()] });
    const websign = await WebSign.connect({ executable: app.executable });
    await websign.close();
    expect(app.argv()).toEqual(["connect"]);
    expect(app.received()[0]).toEqual({
      v: 1,
      id: "n1",
      type: "hello",
      client: { name: "@websign/desktop", version: VERSION },
      protocols: { min: 1, max: 1 },
    });
  });

  it("lets callers name themselves", async () => {
    const app = fake({ rules: [helloRule()] });
    const websign = await WebSign.connect({
      executable: app.executable,
      clientName: "invoice-tool",
      clientVersion: "9.9.9",
    });
    await websign.close();
    expect(app.received()[0]?.client).toEqual({ name: "invoice-tool", version: "9.9.9" });
  });

  it("finds the executable through WEBSIGN_EXECUTABLE", async () => {
    const app = fake({ rules: [helloRule()] });
    process.env.WEBSIGN_EXECUTABLE = app.executable;
    try {
      await (await WebSign.connect()).close();
    } finally {
      delete process.env.WEBSIGN_EXECUTABLE;
    }
    expect(app.argv()).toEqual(["connect"]);
  });

  it("rejects with AppMissing when the executable does not exist", async () => {
    const error = await rejection(WebSign.connect({ executable: "/nonexistent/websign" }));
    expect(error.code).toBe("AppMissing");
  });

  it("rejects with AppMissing when the app exits at once", async () => {
    const app = fake({ rules: [{ on: "start", do: [{ exit: 3 }] }] });
    expect((await rejection(WebSign.connect({ executable: app.executable }))).code).toBe(
      "AppMissing",
    );
  });

  it("rejects with AppMissing when the app exits instead of answering hello", async () => {
    const app = fake({ rules: [{ on: "hello", do: [{ exit: 1 }] }] });
    expect((await rejection(WebSign.connect({ executable: app.executable }))).code).toBe(
      "AppMissing",
    );
  });

  it("rejects with the code of an error reply to hello", async () => {
    const app = fake({
      rules: [
        {
          on: "hello",
          do: [
            send({
              type: "error",
              code: "ClientOutdated",
              message: "upgrade",
              details: { installed: "0", required: "2" },
            }),
          ],
        },
      ],
    });
    const error = await rejection(WebSign.connect({ executable: app.executable }));
    expect(error.code).toBe("ClientOutdated");
    expect(error.message).toBe("upgrade");
    expect(error.details).toEqual({ installed: "0", required: "2" });
  });

  it("rejects with ClientOutdated when the app picks a protocol we do not speak", async () => {
    const app = fake({ rules: [helloRule(2)] });
    expect((await rejection(WebSign.connect({ executable: app.executable }))).code).toBe(
      "ClientOutdated",
    );
    await eventually(() => isAlive(app.pid()) === false);
  });

  it("rejects with Internal when the first reply is not a hello", async () => {
    const app = fake({ rules: [{ on: "hello", do: [send({ type: "done" })] }] });
    expect((await rejection(WebSign.connect({ executable: app.executable }))).code).toBe(
      "Internal",
    );
    await eventually(() => isAlive(app.pid()) === false);
  });

  it("maps an unknown error code to Internal", async () => {
    const app = fake({
      rules: [{ on: "hello", do: [send({ type: "error", code: "Mystery", message: "?" })] }],
    });
    expect((await rejection(WebSign.connect({ executable: app.executable }))).code).toBe(
      "Internal",
    );
  });

  it("works when the reply arrives one byte at a time", async () => {
    const app = fake({
      rules: [
        {
          on: "hello",
          do: [
            {
              sendSplit: {
                chunk: 1,
                message: { type: "hello", protocol: 1, app: { version: "1.4.0" } },
              },
            },
          ],
        },
      ],
    });
    await (await WebSign.connect({ executable: app.executable })).close();
  });
});

describe("misbehaving apps", () => {
  it.each([
    ["garbage bytes", { sendRaw: Buffer.from("not a frame at all").toString("base64") }],
    [
      "a frame of invalid JSON",
      { sendRaw: Buffer.from([2, 0, 0, 0, 0x7b, 0x7b]).toString("base64") },
    ],
    ["an oversized frame header", { header: 1024 * 1024 + 1 }],
    ["a JSON array instead of an envelope", { sendRaw: rawFrame([1, 2]) }],
    ["an envelope without an id", { sendRaw: rawFrame({ v: 1, type: "hello" }) }],
  ])("kills the app and rejects with Internal on %s", async (_name, action) => {
    const app = fake({ rules: [{ on: "hello", do: [action] }] });
    const error = await rejection(WebSign.connect({ executable: app.executable }));
    expect(error.code).toBe("Internal");
    await eventually(() => isAlive(app.pid()) === false);
  });

  it("fails open requests with Internal when the app dies mid-session", async () => {
    const app = fake({ rules: [helloRule(), { on: "status", do: [{ exit: 9 }] }] });
    const websign = await WebSign.connect({ executable: app.executable });
    const error = await rejection(websign.status());
    expect(error.code).toBe("Internal");
    expect(error.message).toMatch(/exited/);
    expect((await rejection(websign.status())).code).toBe("Internal");
    await websign.close();
  });

  it("ignores replies to ids it never used", async () => {
    const app = fake({
      rules: [
        helloRule(),
        {
          on: "status",
          do: [
            { send: { id: "n99", type: "done" } },
            send({ type: "status", app: { version: "1.4.0" }, remembered: true }),
          ],
        },
      ],
    });
    const websign = await WebSign.connect({ executable: app.executable });
    expect((await websign.status()).remembered).toBe(true);
    await websign.close();
  });
});

describe("Session timing and cleanup", () => {
  const identity = { name: "test", version: "0" };

  it("times out a silent app and kills it", async () => {
    const app = fake({ rules: [], ignoreStdinEnd: true });
    const error = await rejection(
      // Long enough for the fake to boot and record its pid on a loaded machine.
      Session.open(app.executable, identity, { helloMs: 1500, closeMs: 100 }),
    );
    expect(error.code).toBe("Timeout");
    await eventually(() => isAlive(app.pid()) === false);
  });

  it("kills an app that ignores the end of stdin after the close timeout", async () => {
    const app = fake({ rules: [helloRule()], ignoreStdinEnd: true });
    const session = await Session.open(app.executable, identity, { helloMs: 2000, closeMs: 150 });
    await session.close();
    expect(isAlive(app.pid())).toBe(false);
  });

  it("close is idempotent and lets a well-behaved app exit on its own", async () => {
    const app = fake({ rules: [helloRule()] });
    const websign = await WebSign.connect({ executable: app.executable });
    await Promise.all([websign.close(), websign.close()]);
    await websign.close();
    expect(isAlive(app.pid())).toBe(false);
  });

  it("rejects requests in flight when closed", async () => {
    const app = fake({ rules: [helloRule()] });
    const websign = await WebSign.connect({ executable: app.executable });
    const pending = rejection(websign.status());
    await websign.close();
    expect((await pending).code).toBe("Internal");
  });

  it("does not leave a listener on process exit after close", async () => {
    const before = process.listenerCount("exit");
    const app = fake({ rules: [helloRule()] });
    await (await WebSign.connect({ executable: app.executable })).close();
    expect(process.listenerCount("exit")).toBe(before);
  });
});

function rawFrame(message: unknown): string {
  const body = Buffer.from(JSON.stringify(message));
  const header = Buffer.alloc(4);
  header.writeUInt32LE(body.length);
  return Buffer.concat([header, body]).toString("base64");
}
