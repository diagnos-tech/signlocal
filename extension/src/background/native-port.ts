/**
 * The port the connection talks through. Chromium and Firefox hand out a
 * real native messaging port (`runtime.connectNative`); Safari has none and
 * gets a port-shaped session over its app extension (`safari-port.ts`), so
 * everything above the port (hello, sharing, idle close, ids) is the same
 * code for every browser.
 */

import { browser } from "wxt/browser";
import type { ErrorCode } from "../generated";
import { NATIVE_HOST } from "../generated";
import { SafariPort } from "./safari-port";

/** What the connection needs of a port; `runtime.Port` has this shape. */
export interface NativePort {
  postMessage(message: object): void;
  disconnect(): void;
  readonly onMessage: { addListener(listener: (message: unknown) => void): void };
  readonly onDisconnect: { addListener(listener: () => void): void };
  /**
   * Why the port closed, read after `onDisconnect` (Firefox fills
   * `message`). `code` is set only when the transport already knows the
   * classification; otherwise it follows from whether the app spoke.
   */
  readonly error?: { readonly message?: string; readonly code?: ErrorCode } | null;
}

/**
 * Safari names its extension pages `safari-web-extension://`. That scheme
 * is fixed by WebKit, unlike the user agent (which Safari lets people and
 * other browsers imitate) and unlike the build target (one test build runs
 * everywhere).
 */
export function runsInSafari(): boolean {
  return browser.runtime.getURL("/").startsWith("safari-web-extension:");
}

/** Opens a port to the app; may throw like `connectNative` for a missing host. */
export function openNativePort(): NativePort {
  if (runsInSafari()) return new SafariPort();
  return browser.runtime.connectNative(NATIVE_HOST);
}
