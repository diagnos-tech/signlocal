/**
 * What an e2e run needs, from the environment set up by CI (or a developer):
 * the e2e app build, the unpacked extension, the software keys and where
 * screenshots go.
 */

/** The e2e environment. */
export interface E2eEnvironment {
  /** `websign` built with `--features e2e` (WEBSIGN_E2E_APP). */
  readonly app: string;
  /** Unpacked chromium extension (WEBSIGN_E2E_EXTENSION, default extension/.output/chrome-mv3). */
  readonly extension: string;
  /** Screenshot folder (WEBSIGN_E2E_SCREENSHOTS). */
  readonly screenshots: string;
  /** Browser executable to use instead of Playwright's (WEBSIGN_E2E_BROWSER). */
  readonly browser?: string;
}

/** Reads the environment, failing with a clear message when something is missing. */
export function environment(): E2eEnvironment {
  throw new Error("unimplemented: testing.md §E2E");
}
