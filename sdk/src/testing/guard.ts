/**
 * Keeps the fake out of production: it signs with published keys, so on a
 * real site it would let anyone "sign" as a fictional person. Local origins
 * pass; anything else needs an explicit `allowAnyOrigin`.
 */

const LOCAL_HOST = /^(localhost|127(\.\d{1,3}){3}|\[::1\]|0\.0\.0\.0)$/;
const TEST_SUFFIX = /\.(localhost|test|example|invalid|local)$/;

/** Whether `location` is a developer machine or a test DOM. */
export function isTestOrigin(
  location: Pick<Location, "protocol" | "hostname"> | undefined,
): boolean {
  if (location === undefined) return true;
  const { protocol, hostname } = location;
  if (["file:", "about:", "data:", "blob:"].includes(protocol) || hostname === "") return true;
  return LOCAL_HOST.test(hostname) || TEST_SUFFIX.test(hostname);
}

/** @throws {Error} on a non-local origin unless `allowAnyOrigin`. */
export function assertTestOrigin(win: Window, allowAnyOrigin: boolean): void {
  const location = (win as { location?: Location }).location;
  if (allowAnyOrigin || isTestOrigin(location)) return;
  throw new Error(
    `installFakeWebSign() refused to run on ${location?.origin}: the fake signs with public test keys. ` +
      "Use it only in tests and local development, or pass { allowAnyOrigin: true } for a staging server.",
  );
}

// The SDK itself never writes to the console; this entry does, on purpose, so
// a fake that slipped into a build is noticed at once.
export function warnInstalled(): void {
  console.warn(
    "[@websign/sdk/testing] A FAKE SignLocal extension is active on this page. Its signatures use " +
      "public test keys and prove nothing. Never ship @websign/sdk/testing to production.",
  );
}

export function warnRealExtension(): void {
  console.warn(
    "[@websign/sdk/testing] The real SignLocal extension also answers on this page; answers will mix. " +
      "Disable the extension for this site, or do not install the fake.",
  );
}
