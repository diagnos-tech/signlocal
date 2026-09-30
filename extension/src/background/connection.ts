/**
 * The native messaging port: opened on demand, `hello` first, kept open
 * while used, closed after EXTENSION_IDLE_CLOSE (60 s) without traffic.
 * Promotes the proven logic of docs/prototypes/kit/extension/native-host.js.
 */

import type { AppEnvelope, ClientEnvelope, HelloReply } from "../generated";

/** A live connection to the app. */
export interface Connection {
  readonly hello: HelloReply;
  send(message: ClientEnvelope): void;
  onMessage(listener: (message: AppEnvelope) => void): () => void;
  /** Resolves when the port closes; `heard` = the app answered at least once. */
  readonly closed: Promise<{ readonly heard: boolean; readonly reason: string }>;
}

/** The current connection, opening it (and saying hello) if needed. */
export function connect(): Promise<Connection> {
  throw new Error("unimplemented: SPEC.md §2.1");
}
