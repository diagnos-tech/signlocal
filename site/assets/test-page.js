// The test-signature page: detects the extension and the app, signs a fixed
// sample text through the SDK, checks the signature in the browser and builds
// a summary without personal data. Nothing is sent to any server.
import { errorText, fingerprint, installUrl, onChange, sign, status, WebSignError } from "./websign-sdk.js";
import { digestOf, verifySignature } from "./test-crypto.js";

const SAMPLE = "WebeSign test message. This is not a document.";
const message = new TextEncoder().encode(SAMPLE);
const $ = (id) => document.getElementById(id);
const subtle = globalThis.crypto && globalThis.crypto.subtle;

let current = null; // last status
let busy = false;
let outcome = null; // last sign attempt: { certificate, code, result, check, hash, digest, error }

function setupState(s) {
  if (s === null) return "checking";
  if (!s.extension.installed) return "extension-missing";
  if (!s.app.installed) return "app-missing";
  if (s.app.outdated) return "outdated";
  return "ready";
}

function renderSetup() {
  const state = setupState(current);
  for (const el of document.querySelectorAll("#setup [data-state]")) el.hidden = el.dataset.state !== state;
  $("sign").disabled = state !== "ready" || !subtle || busy;
  $("v-ext").textContent = current?.extension.version ?? "-";
  $("v-app").textContent = current?.app.version ?? "-";
  $("install-ext").href = installUrl();
}

function fill(text, details) {
  return text.replace("{installed}", details?.installed ?? "?").replace("{required}", details?.required ?? "?");
}

function keyText(key) {
  return key.type === "RSA" ? `RSA ${key.bits}` : `EC ${key.curve}`;
}

function renderIdenticon(code) {
  const icon = $("r-icon");
  icon.replaceChildren(
    ...code.cells.map((on) => {
      const cell = document.createElement("i");
      if (on) cell.className = "on";
      return cell;
    }),
  );
  icon.style.setProperty("--id", `var(--ws-id-${code.colorIndex})`);
}

function summaryText() {
  const lines = ["WebeSign test"];
  lines.push(`setup: ${setupState(current)}`);
  if (current?.extension.version) lines.push(`extension: ${current.extension.version}`);
  if (current?.app.version) lines.push(`app: ${current.app.version}`);
  if (outcome) {
    lines.push(`result: ${outcome.error || outcome.check === "invalid" ? "failure" : "success"}`);
    lines.push(`error: ${outcome.error ?? "none"}`);
    lines.push(`hash: ${outcome.hash}`);
    if (outcome.result) {
      lines.push(`algorithm: ${outcome.result.algorithm}`);
      lines.push(`key: ${keyText(outcome.result.certificate.key)}`);
      lines.push(`browser check: ${outcome.check === "unsupported" ? "not verified" : outcome.check}`);
    }
  }
  return lines.join("\n");
}

function renderOutcome() {
  $("result").hidden = outcome === null;
  if (outcome === null) return;
  const { result, error, details } = outcome;
  $("result-ok").hidden = !result;
  $("result-error").hidden = !error;
  if (result) {
    $("r-name").textContent = result.certificate.displayName;
    $("r-alg").textContent = `${result.algorithm} · ${result.hash}`;
    $("r-key").textContent = keyText(result.certificate.key);
    const code = fingerprint(outcome.digest);
    $("r-code").textContent = code.text;
    renderIdenticon(code);
    $("ok-title").hidden = outcome.check === "invalid";
    $("bad-title").hidden = outcome.check !== "invalid";
    $("result-ok").classList.toggle("warn", outcome.check === "invalid");
    for (const name of ["verified", "invalid", "unsupported"]) $(`v-${name}`).hidden = outcome.check !== name;
  } else {
    const texts = errorText(error, document.documentElement.lang);
    $("e-title").textContent = texts ? fill(texts.title, details) : $("generic-error").textContent;
    $("e-body").textContent = texts ? fill(texts.body, details) : "";
    $("e-code").textContent = error;
  }
  $("summary").textContent = summaryText();
}

async function runTest() {
  const hash = $("hash").value;
  let digest = null;
  busy = true;
  renderSetup();
  $("waiting").hidden = false;
  try {
    const result = await sign({
      hash,
      prepare: async () => {
        digest = await digestOf(subtle, hash, message);
        return digest;
      },
    });
    const check = await verifySignature(subtle, {
      certificateDer: result.certificate.der,
      algorithm: result.algorithm,
      hash: result.hash,
      signature: result.signature,
      message,
    });
    outcome = { hash, result, digest, check };
  } catch (error) {
    const code = error instanceof WebSignError ? error.code : "Internal";
    outcome = { hash, error: code, details: error instanceof WebSignError ? error.details : undefined };
  }
  busy = false;
  $("waiting").hidden = true;
  renderSetup();
  renderOutcome();
  $("result").scrollIntoView({ block: "nearest" });
}

async function copySummary() {
  let note = $("copied-text").textContent;
  try {
    await navigator.clipboard.writeText(summaryText());
  } catch {
    note = $("copy-failed-text").textContent;
  }
  $("copied").textContent = note;
}

async function refresh() {
  current = null;
  renderSetup();
  current = await status();
  renderSetup();
  if (outcome) $("summary").textContent = summaryText();
}

$("sample").textContent = SAMPLE;
$("sign").addEventListener("click", runTest);
$("copy").addEventListener("click", copySummary);
$("recheck").addEventListener("click", refresh);
document.addEventListener("websign:i18n", () => {
  renderSetup();
  renderOutcome();
  $("copied").textContent = "";
});
if (!subtle) $("nocrypto").hidden = false;
onChange((s) => {
  current = s;
  renderSetup();
});
renderSetup();
refresh();
