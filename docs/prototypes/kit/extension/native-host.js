"use strict";

// One native messaging port, opened on demand and kept while it is used.
//
// Starting the host costs a process launch (and, for PKCS#11, a driver load),
// so the port stays open between requests and is closed after a minute of
// silence. A dead port is replaced on the next request; requests in flight
// when it dies get a typed error instead of hanging.
//
// Exposes `globalThis.WebsignHost.request(type, fields)`, which always
// resolves (never rejects) with the host's reply or a local error reply of
// the same shape: `{ ok: false, error: { code, message } }`.

(() => {
  const api = globalThis.browser ?? globalThis.chrome;
  const { nativeHost } = globalThis.WEBSIGN_CONFIG;

  const PROTOCOL_VERSION = 1;
  const IDLE_CLOSE_MS = 60_000;
  // Signing waits for a person, so it gets far longer than the others.
  const TIMEOUT_MS = { ping: 15_000, list: 30_000, sign: 120_000 };

  /** @type {{ port: chrome.runtime.Port, heard: boolean } | null} */
  let connection = null;
  let idleTimer = null;
  let nextId = 1;
  const pending = new Map(); // id -> { resolve, timer }

  const failure = (code, message) => ({
    v: PROTOCOL_VERSION,
    ok: false,
    error: { code, message },
  });

  function connect() {
    if (connection) return connection;
    const port = api.runtime.connectNative(nativeHost);
    const current = { port, heard: false };
    port.onMessage.addListener((message) => {
      current.heard = true;
      onMessage(message);
    });
    port.onDisconnect.addListener(() => onDisconnect(current));
    connection = current;
    return current;
  }

  function onMessage(message) {
    const id = message && typeof message.id === "string" ? message.id : null;
    const entry = id === null ? undefined : pending.get(id);
    if (!entry) return; // Unsolicited or late: nothing is waiting for it.
    pending.delete(id);
    clearTimeout(entry.timer);
    entry.resolve(message);
  }

  function onDisconnect(closed) {
    // Reading lastError is what stops the browser logging "unchecked".
    const reason =
      api.runtime.lastError?.message ?? closed.port.error?.message ?? "the host closed the connection";
    if (connection === closed) connection = null;
    clearTimeout(idleTimer);
    const reply = closed.heard
      ? failure("host_disconnected", reason)
      : failure("host_unavailable", reason);
    for (const [id, entry] of pending) {
      pending.delete(id);
      clearTimeout(entry.timer);
      entry.resolve(reply);
    }
  }

  function armIdleTimer() {
    clearTimeout(idleTimer);
    idleTimer = setTimeout(() => {
      if (pending.size > 0) return armIdleTimer();
      const closing = connection;
      connection = null;
      closing?.port.disconnect();
    }, IDLE_CLOSE_MS);
  }

  function post(id, type, fields) {
    const message = { v: PROTOCOL_VERSION, id, type, ...fields };
    try {
      connect().port.postMessage(message);
    } catch {
      // The port may have died since the last request: retry once on a fresh one.
      connection = null;
      connect().port.postMessage(message);
    }
  }

  function request(type, fields = {}) {
    return new Promise((resolve) => {
      const id = String(nextId++);
      const timer = setTimeout(() => {
        pending.delete(id);
        resolve(failure("timeout", `no reply to "${type}" from the native host`));
      }, TIMEOUT_MS[type] ?? 30_000);
      pending.set(id, { resolve, timer });
      try {
        post(id, type, fields);
      } catch (error) {
        pending.delete(id);
        clearTimeout(timer);
        resolve(failure("host_unavailable", String(error?.message ?? error)));
        return;
      }
      armIdleTimer();
    });
  }

  globalThis.WebsignHost = Object.freeze({ request });
})();
