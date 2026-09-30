/**
 * The slice of Playwright's API that screenshots.ts uses. Playwright is a
 * dependency of the e2e package, not of the extension, so its own types do
 * not resolve here; this keeps the script type-checked without a new dependency.
 */

/** A region of the page in CSS px. */
export interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Page {
  setViewportSize(size: { width: number; height: number }): Promise<void>;
  emulateMedia(options: { colorScheme?: string; reducedMotion?: string }): Promise<void>;
  goto(url: string): Promise<unknown>;
  waitForSelector(selector: string, options?: { timeout?: number }): Promise<unknown>;
  waitForTimeout(ms: number): Promise<void>;
  evaluate<T>(run: () => T): Promise<T>;
  screenshot(options: { path: string; clip: Box }): Promise<unknown>;
  close(): Promise<void>;
}

export interface Context {
  newPage(): Promise<Page>;
  on(event: "close", listener: () => void): void;
  close(): Promise<void>;
}

export interface Chromium {
  launchPersistentContext(
    profile: string,
    options: {
      channel?: string;
      headless?: boolean;
      locale?: string;
      deviceScaleFactor?: number;
      env?: Record<string, string | undefined>;
      args?: string[];
    },
  ): Promise<Context>;
}
