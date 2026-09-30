/**
 * A minimal WebDriver BiDi client over Node's built-in WebSocket: commands
 * and their replies, nothing else (the suite needs no events). Firefox
 * speaks BiDi natively, so driving it needs no geckodriver and no extra
 * package; Playwright's own Firefox is a patched build that cannot load
 * extensions.
 */

/** A BiDi command failed. */
export class BidiError extends Error {}

interface Reply {
  readonly id?: number;
  readonly type: "success" | "error" | "event";
  readonly result?: unknown;
  readonly error?: string;
  readonly message?: string;
}

/** One BiDi session. */
export class Bidi {
  private next = 0;
  private readonly waiting = new Map<number, (reply: Reply) => void>();

  /** `browserVersion` of the session's capabilities. */
  version = "unknown";

  private constructor(private readonly socket: WebSocket) {
    socket.addEventListener("message", (event) => {
      const reply = JSON.parse(String(event.data)) as Reply;
      if (reply.id === undefined) return;
      this.waiting.get(reply.id)?.(reply);
      this.waiting.delete(reply.id);
    });
    socket.addEventListener("close", () => {
      for (const done of this.waiting.values()) {
        done({ type: "error", error: "closed", message: "the browser closed the connection" });
      }
      this.waiting.clear();
    });
  }

  /** Connects to `url` (the `ws://host:port` Firefox prints) and starts a session. */
  static async connect(url: string): Promise<Bidi> {
    const socket = new WebSocket(`${url}/session`);
    await new Promise<void>((open, fail) => {
      socket.addEventListener("open", () => open(), { once: true });
      socket.addEventListener("error", () => fail(new BidiError(`cannot reach ${url}`)), {
        once: true,
      });
    });
    const bidi = new Bidi(socket);
    const session = await bidi.send<{ capabilities: { browserVersion?: string } }>("session.new", {
      capabilities: {},
    });
    bidi.version = session.capabilities.browserVersion ?? "unknown";
    return bidi;
  }

  /** Sends `method` and resolves with its result, or rejects with the browser's error. */
  send<T = unknown>(method: string, params: object): Promise<T> {
    const id = ++this.next;
    return new Promise<T>((resolve, reject) => {
      this.waiting.set(id, (reply) => {
        if (reply.type === "success") resolve(reply.result as T);
        else reject(new BidiError(`${method}: ${reply.error}: ${reply.message}`));
      });
      this.socket.send(JSON.stringify({ id, method, params }));
    });
  }

  /** Ends the session (closing the browser is the caller's job). */
  async close(): Promise<void> {
    await this.send("session.end", {}).catch(() => undefined);
    this.socket.close();
  }
}
