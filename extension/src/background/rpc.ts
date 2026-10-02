/** One request, one answer, over the shared connection (for the extension's own asks). */

import type { AppEnvelope, ClientMessage } from "../generated";
import { PROTOCOL_VERSION } from "../shared/limits";
import { AppError } from "./app-error";
import type { Connection } from "./connection";

let counter = 0;

/** Sends `message` and resolves with the app's first message for it, or fails at `timeoutMs`. */
export function ask(
  conn: Connection,
  message: ClientMessage,
  timeoutMs: number,
): Promise<AppEnvelope> {
  // Tab-borne ids have two dots, so "ext.<n>" can never collide with them.
  counter += 1;
  const id = `ext.${counter}`;
  return new Promise((resolve, reject) => {
    const stop = conn.onMessage((reply) => {
      if (reply.id !== id) return;
      clearTimeout(timer);
      stop();
      resolve(reply);
    });
    const timer = setTimeout(() => {
      stop();
      reject(new AppError("Timeout", "the app did not answer"));
    }, timeoutMs);
    try {
      conn.send({ ...message, v: PROTOCOL_VERSION, id } as Parameters<Connection["send"]>[0]);
    } catch (error) {
      clearTimeout(timer);
      stop();
      reject(error);
    }
  });
}
