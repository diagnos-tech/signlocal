/**
 * In-memory stand-ins for the WebExtension APIs the background and content
 * modules use. Installed on `globalThis.browser`/`chrome`; tests that mock
 * "wxt/browser" hand out the same object. The object is a process-wide
 * singleton: `vi.resetModules()` re-evaluates this file, and a second copy
 * would leave the test holding a different fake than the module under test.
 */

import { vi } from "vitest";

/** A WebExtension event: listeners plus a test-side `emit`. */
export class FakeEvent<A extends unknown[] = unknown[]> {
  readonly listeners = new Set<(...args: A) => unknown>();
  addListener = (listener: (...args: A) => unknown): void => {
    this.listeners.add(listener);
  };
  removeListener = (listener: (...args: A) => unknown): void => {
    this.listeners.delete(listener);
  };
  hasListener = (listener: (...args: A) => unknown): boolean => this.listeners.has(listener);
  emit(...args: A): void {
    for (const listener of [...this.listeners]) listener(...args);
  }
  clear(): void {
    this.listeners.clear();
  }
}

/** A native messaging port; `sent` records what the extension posted. */
export class FakePort {
  readonly sent: Array<Record<string, unknown>> = [];
  readonly onMessage = new FakeEvent<[unknown]>();
  readonly onDisconnect = new FakeEvent<[]>();
  disconnected = false;
  disconnect = vi.fn(() => {
    this.disconnected = true;
  });
  postMessage = vi.fn((message: Record<string, unknown>) => {
    this.sent.push(message);
  });
  /** The app says something. */
  receive(message: unknown): void {
    this.onMessage.emit(message);
  }
  /** The browser closes the port (host missing, crashed, exited). */
  drop(): void {
    this.disconnected = true;
    this.onDisconnect.emit();
  }
}

function createFakeBrowser() {
  const fake = {
    ports: [] as FakePort[],
    connectNativeError: null as Error | null,
    /** The scheme of extension URLs: `safari-web-extension` makes the background use the relay. */
    urlScheme: "chrome-extension",
    manifestVersion: "0.1.0",
    runtime: {
      id: "websign-test-extension",
      onMessage: new FakeEvent<[unknown, unknown, (response?: unknown) => void]>(),
      onStartup: new FakeEvent<[]>(),
      onInstalled: new FakeEvent<[unknown]>(),
      lastError: undefined as { message: string } | undefined,
      sendMessage: vi.fn(async (_message: unknown): Promise<unknown> => undefined),
      getManifest: () => ({ version: fake.manifestVersion }),
      getPlatformInfo: vi.fn(async () => ({ os: "linux" })),
      getURL: (path: string) => `${fake.urlScheme}://websign-test-extension${path}`,
      sendNativeMessage: vi.fn(
        async (_application: string, _message: unknown): Promise<unknown> => undefined,
      ),
      connectNative: vi.fn((_host: string): FakePort => {
        if (fake.connectNativeError) throw fake.connectNativeError;
        const port = new FakePort();
        fake.ports.push(port);
        return port;
      }),
    },
    tabs: {
      onRemoved: new FakeEvent<[number]>(),
      onUpdated: new FakeEvent<[number, { status?: string; url?: string }, unknown]>(),
      sendMessage: vi.fn(
        async (_tabId: number, _message: unknown, _options?: unknown) => undefined,
      ),
    },
    i18n: {
      getMessage: vi.fn((key: string, _substitutions?: string | string[]) => key),
      getUILanguage: () => "en-US",
    },
    action: {
      setBadgeText: vi.fn(async (_details: { text: string }) => {}),
      setBadgeBackgroundColor: vi.fn(async (_details: { color: string }) => {}),
    },
    reset(): void {
      this.ports = [];
      this.connectNativeError = null;
      this.urlScheme = "chrome-extension";
      this.runtime.sendNativeMessage.mockReset();
      this.runtime.sendNativeMessage.mockImplementation(async () => undefined);
      this.manifestVersion = "0.1.0";
      this.runtime.onMessage.clear();
      this.runtime.onStartup.clear();
      this.runtime.onInstalled.clear();
      this.tabs.onRemoved.clear();
      this.tabs.onUpdated.clear();
      this.runtime.connectNative.mockClear();
      this.i18n.getMessage.mockReset();
      this.i18n.getMessage.mockImplementation((key: string) => key);
      this.runtime.sendMessage.mockReset();
      this.runtime.sendMessage.mockImplementation(async () => undefined);
      this.runtime.getPlatformInfo.mockReset();
      this.runtime.getPlatformInfo.mockImplementation(async () => ({ os: "linux" }));
      this.tabs.sendMessage.mockReset();
      this.tabs.sendMessage.mockImplementation(async () => undefined);
      this.action.setBadgeText.mockClear();
      this.action.setBadgeBackgroundColor.mockClear();
    },
  };
  return fake;
}

const shared = globalThis as { __websignFakeBrowser?: ReturnType<typeof createFakeBrowser> };
shared.__websignFakeBrowser ??= createFakeBrowser();

/** The fake `browser` namespace. Call `reset()` between tests. */
export const fakeBrowser = shared.__websignFakeBrowser;

Object.assign(globalThis, { browser: fakeBrowser, chrome: fakeBrowser });
