// The test page: detects the extension and the app, signs a fixed sample text through the SDK,
// checks the signature in the browser and builds a summary without personal data. Nothing is
// sent anywhere.

import { fingerprint, isSignlocalError, onChange, sign, status } from "@signlocal/sdk";
import type { MessageKey } from "../../generated/message-keys.ts";
import { detect } from "../../platform/detect.ts";
import { readNavigator } from "../../platform/navigator.ts";
import { installTestHooks } from "../../wizard/test-hooks.ts";
import { readStrings } from "../strings.ts";
import { digestOf, verifySignature } from "./crypto.ts";
import {
  renderFailure,
  renderInstallLinks,
  renderSetup,
  renderSuccess,
  renderSummary,
} from "./render.ts";
import { type Outcome, SAMPLE_TEXT, type SetupStatus, summaryText } from "./state.ts";

export const STRINGS = [
  "test.copied",
  "test.copy_failed",
  "test.generic_error",
] as const satisfies readonly MessageKey[];

const message = new TextEncoder().encode(SAMPLE_TEXT);
const subtle = globalThis.crypto?.subtle;
const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

export async function initTestPage(): Promise<void> {
  await installTestHooks(new URLSearchParams(location.search).get("sdk-fake"));
  const strings = readStrings("test", STRINGS);
  let current: SetupStatus | null = null;
  let outcome: Outcome | null = null;
  let busy = false;

  const redraw = () => {
    renderSetup(current, busy, subtle !== undefined);
    if (outcome) renderSummary(summaryText(current, outcome));
  };

  const { platform, browser } = detect(readNavigator(navigator));
  const params = new URLSearchParams();
  if (platform === "windows" || platform === "macos" || platform === "linux")
    params.set("os", platform);
  if (browser) params.set("browser", browser);
  const base = document.getElementById("install-ext")?.getAttribute("href") ?? "";
  renderInstallLinks(params.size > 0 ? `${base}?${params}` : base);

  async function run(): Promise<void> {
    const hash = $<HTMLSelectElement>("hash").value;
    let digest: Uint8Array<ArrayBuffer> | null = null;
    busy = true;
    redraw();
    $("waiting").hidden = false;
    try {
      const result = await sign({
        hash: hash as "SHA-256",
        prepare: async () => {
          digest = await digestOf(subtle as SubtleCrypto, hash, message);
          return digest;
        },
      });
      const check = await verifySignature(subtle as SubtleCrypto, {
        certificateDer: result.certificate.der,
        algorithm: result.algorithm,
        hash: result.hash,
        signature: result.signature,
        message,
      });
      outcome = { hash, algorithm: result.algorithm, key: result.certificate.key, check };
      renderSuccess({
        certificate: result.certificate,
        algorithm: result.algorithm,
        hash: result.hash,
        code: fingerprint(digest ?? result.digest),
        check,
      });
    } catch (error) {
      const code = isSignlocalError(error) ? error.code : "Internal";
      outcome = { hash, error: code };
      renderFailure(
        code,
        isSignlocalError(error) ? error : undefined,
        document.documentElement.lang,
        strings["test.generic_error"],
      );
    }
    busy = false;
    $("waiting").hidden = true;
    redraw();
    $("result").scrollIntoView({ block: "nearest" });
  }

  async function copy(): Promise<void> {
    let note = strings["test.copied"];
    try {
      await navigator.clipboard.writeText(summaryText(current, outcome));
    } catch {
      note = strings["test.copy_failed"];
    }
    $("copied").textContent = note;
  }

  const refresh = async () => {
    current = null;
    redraw();
    current = await status();
    redraw();
  };

  $("sign").addEventListener("click", run);
  $("copy").addEventListener("click", copy);
  $("recheck").addEventListener("click", refresh);
  if (!subtle) $("nocrypto").hidden = false;
  onChange((next) => {
    current = next;
    redraw();
  });
  redraw();
  void refresh();
}
