/**
 * A Safari app extension relay in memory (`safari/SPEC.md` §1–2), driven
 * through the fake `runtime.sendNativeMessage`. Each session plays a host:
 * `answer` decides what it says to each message, `say` lets a test make it
 * speak later, `exit` ends it. Polls park until the host speaks, `wait`
 * passes, or a newer request supersedes them, as in the real relay.
 */

import { helloEnvelope } from "./protocol";

type Reply = Record<string, unknown>;
type Envelope = { id: string; type: string } & Record<string, unknown>;

interface Session {
  readonly id: string;
  readonly received: Envelope[];
  pending: Envelope[];
  open: boolean;
  parked: { reply: (value: Reply) => void; timer: ReturnType<typeof setTimeout> } | null;
}

let counter = 0;
const uuid = () =>
  `00000000-0000-4000-8000-${(++counter).toString(16).toUpperCase().padStart(12, "0")}`;

export class FakeAppex {
  readonly sessions = new Map<string, Session>();
  readonly requests: Reply[] = [];
  /** Polls the relay is holding right now, and the most it ever held at once. */
  parkedNow = 0;
  maxParked = 0;
  /** Replaces the whole reply to `open` (an error, a malformed value). */
  openReply: Reply | null = null;
  /** What the host answers to each message; by default hello gets hello. */
  answer: (message: Envelope) => Envelope[] = (message) =>
    message.type === "hello" ? [helloEnvelope(message.id) as Envelope] : [];

  /** The function to install as `sendNativeMessage`. */
  readonly handle = async (_application: string, request: unknown): Promise<unknown> => {
    const call = request as Reply;
    this.requests.push(call);
    if (call.op === "open") return this.openReply ?? this.opened();
    const session = this.sessions.get(call.session as string);
    if (session === undefined) return { relay: 1, error: "NoSession" };
    this.wake(session, false);
    switch (call.op) {
      case "send": {
        const message = call.message as Envelope;
        session.received.push(message);
        session.pending.push(...this.answer(message));
        return this.drain(session);
      }
      case "poll":
        if (session.pending.length > 0 || !session.open) return this.drain(session);
        return this.park(session, call.wait as number);
      case "close":
        this.sessions.delete(session.id);
        return { relay: 1, session: session.id, messages: [], open: false };
      default:
        return { relay: 1, error: "BadRequest" };
    }
  };

  /** The only session, for tests that open one. */
  get session(): Session {
    const [only] = this.sessions.values();
    if (only === undefined) throw new Error("no session is open");
    return only;
  }

  /** The host says something on its own (a reply that took a while). */
  say(session: Session, ...messages: Envelope[]): void {
    session.pending.push(...messages);
    this.wake(session, true);
  }

  /** The host process ends. */
  exit(session: Session): void {
    session.open = false;
    this.wake(session, true);
  }

  ops(): string[] {
    return this.requests.map((request) => request.op as string);
  }

  private opened(): Reply {
    const session: Session = { id: uuid(), received: [], pending: [], open: true, parked: null };
    this.sessions.set(session.id, session);
    return { relay: 1, session: session.id, messages: [], open: true };
  }

  private drain(session: Session): Reply {
    const messages = session.pending;
    session.pending = [];
    if (!session.open) this.sessions.delete(session.id);
    return { relay: 1, session: session.id, messages, open: session.open };
  }

  private park(session: Session, wait: number): Promise<Reply> {
    this.parkedNow += 1;
    this.maxParked = Math.max(this.maxParked, this.parkedNow);
    return new Promise((reply) => {
      const timer = setTimeout(() => this.wake(session, true), wait);
      session.parked = { reply, timer };
    });
  }

  /** Answers the parked poll: with what is pending, or empty when superseded. */
  private wake(session: Session, withPending: boolean): void {
    const parked = session.parked;
    if (parked === null) return;
    session.parked = null;
    this.parkedNow -= 1;
    clearTimeout(parked.timer);
    parked.reply(
      withPending
        ? this.drain(session)
        : { relay: 1, session: session.id, messages: [], open: true },
    );
  }
}
