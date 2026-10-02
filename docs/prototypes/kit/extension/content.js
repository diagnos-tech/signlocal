"use strict";

// Runs in every allowed page. Announces the extension and relays requests
// between the page and the background script. The page is untrusted: only
// same-window messages of a known shape are forwarded, rebuilt field by field,
// and the background validates them again.

(() => {
  const api = globalThis.browser ?? globalThis.chrome;
  const version = api.runtime.getManifest().version;
  const REQUEST_TYPES = new Set(["status", "ping", "list", "sign"]);
  const SIGN_FIELDS = ["fingerprint", "hash", "algorithm", "digest"];

  const post = (message) => window.postMessage({ source: "websign-extension", ...message }, window.location.origin);
  const announce = () => post({ type: "announce", version });

  const isRecord = (value) => typeof value === "object" && value !== null && !Array.isArray(value);

  /** A clean copy of the request, or null when it is not one of ours. */
  function sanitize(request) {
    if (!isRecord(request) || !REQUEST_TYPES.has(request.type)) return null;
    if (request.type !== "sign") return { type: request.type };
    const clean = { type: "sign" };
    for (const field of SIGN_FIELDS) {
      if (typeof request[field] !== "string" || request[field].length > 256) return null;
      clean[field] = request[field];
    }
    return clean;
  }

  async function relay(id, request) {
    const clean = sanitize(request);
    if (!clean) {
      post({ type: "response", id, response: { ok: false, error: { code: "bad_request", message: "malformed request" } } });
      return;
    }
    let response;
    try {
      response = await api.runtime.sendMessage({ kind: "websign-request", request: clean });
    } catch (error) {
      response = { ok: false, error: { code: "extension_unavailable", message: String(error?.message ?? error) } };
    }
    post({ type: "response", id, response });
  }

  window.addEventListener("message", (event) => {
    if (event.source !== window || event.origin !== window.location.origin) return;
    const data = event.data;
    if (!isRecord(data) || data.source !== "websign-page") return;
    if (data.type === "hello") return announce();
    if (data.type === "request" && typeof data.id === "string" && data.id.length <= 64) {
      void relay(data.id, data.request);
    }
  });

  announce();
  window.addEventListener("load", announce);
})();
