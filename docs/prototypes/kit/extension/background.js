"use strict";

// Service worker / event page: the only part of the extension that talks to
// the native host. Pages reach it through content.js, which cannot be trusted
// (it shares a process with the page), so everything is validated here and
// the site is taken from the browser's `sender`, never from the payload.

if (typeof importScripts === "function") {
  importScripts("config.js", "native-host.js");
}

const api = globalThis.browser ?? globalThis.chrome;
const { minAppVersion, pages } = globalThis.WEBSIGN_CONFIG;
const host = globalThis.WebsignHost;

const HASHES = new Set(["SHA-256", "SHA-384", "SHA-512"]);
const ALGORITHMS = new Set(["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"]);
const FINGERPRINT = /^[0-9a-fA-F: ]{64,95}$/;
const BASE64 = /^[A-Za-z0-9+/]{1,128}={0,2}$/;

const localError = (code, message) => ({ ok: false, error: { code, message } });

/** "0.10.2" -> [0, 10, 2]; anything else -> null. */
const parseVersion = (text) =>
  /^\d+(\.\d+){0,2}$/.test(String(text)) ? String(text).split(".").map(Number) : null;

function isOlder(version, minimum) {
  const a = parseVersion(version);
  const b = parseVersion(minimum);
  if (!a || !b) return true;
  for (let i = 0; i < 3; i += 1) {
    const diff = (a[i] ?? 0) - (b[i] ?? 0);
    if (diff !== 0) return diff < 0;
  }
  return false;
}

function clientInfo(reason) {
  return {
    extension_version: api.runtime.getManifest().version,
    user_agent: navigator.userAgent,
    reason,
  };
}

/**
 * Pings the app and classifies what came back. Every ping also tells the app
 * that this browser has the extension, which is how it learns about every
 * extension connection.
 */
async function status(reason) {
  const reply = await host.request("ping", { client: clientInfo(reason) });
  if (reply.ok) {
    const outdated = isOlder(reply.app?.version, minAppVersion);
    return {
      state: outdated ? "outdated" : "ready",
      minAppVersion,
      app: reply.app,
      os: reply.os,
      arch: reply.arch,
      launch: reply.launch,
    };
  }
  const code = reply.error?.code;
  const state =
    code === "host_unavailable" ? "not-installed" : code === "unsupported_version" ? "outdated" : "error";
  return { state, minAppVersion, error: reply.error };
}

/** The site that asked, as the browser reports it. */
function siteOf(sender) {
  try {
    return new URL(sender.url ?? "").origin;
  } catch {
    return null;
  }
}

const allowedSite = (site) =>
  site !== null && pages.some((pattern) => pattern.replace("/*", "") === site.replace(/:\d+$/, ""));

const isText = (value, max) => typeof value === "string" && value.length <= max;

const handlers = {
  status: () => status("page"),
  ping: () => host.request("ping", { client: clientInfo("page") }),
  list: (_request, site) => host.request("list", { origin: site }),
  sign(request, site) {
    const { fingerprint, hash, algorithm, digest } = request;
    if (
      !isText(fingerprint, 95) ||
      !FINGERPRINT.test(fingerprint) ||
      !HASHES.has(hash) ||
      !ALGORITHMS.has(algorithm) ||
      !isText(digest, 132) ||
      !BASE64.test(digest)
    ) {
      return localError("bad_request", "malformed sign request");
    }
    return host.request("sign", { origin: site, fingerprint, hash, algorithm, digest });
  },
};

async function onPageRequest(message, sender) {
  if (sender.id !== api.runtime.id || !sender.tab) {
    return localError("forbidden", "requests must come from a page of this extension");
  }
  const site = siteOf(sender);
  if (!allowedSite(site)) return localError("forbidden", "this site is not allowed");
  const type = message?.request?.type;
  const handler = Object.hasOwn(handlers, type) ? handlers[type] : null;
  if (!handler) return localError("bad_request", "unknown request type");
  return handler(message.request, site);
}

api.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message?.kind !== "websign-request") return false;
  onPageRequest(message, sender).then(sendResponse, (error) =>
    sendResponse(localError("internal", String(error?.message ?? error))),
  );
  return true; // The reply is asynchronous.
});

api.runtime.onStartup.addListener(() => void status("startup"));
api.runtime.onInstalled.addListener(() => void status("installed"));
