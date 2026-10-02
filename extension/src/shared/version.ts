/** Numeric dotted versions ("1.10.2"); anything else compares as older. */

const NUMERIC = /^\d+(\.\d+){0,2}$/;

/** [major, minor, patch] with missing parts as 0, or null when not numeric. */
function parse(text: string): readonly [number, number, number] | null {
  if (!NUMERIC.test(text)) return null;
  const [major = 0, minor = 0, patch = 0] = text.split(".").map(Number);
  return [major, minor, patch];
}

/**
 * Whether `version` is older than `minimum`. Unparseable input counts as
 * older: an app we cannot identify must not be trusted with a signature.
 */
export function isOlder(version: string, minimum: string): boolean {
  const have = parse(version);
  const need = parse(minimum);
  if (have === null || need === null) return true;
  for (let i = 0; i < 3; i += 1) {
    const difference = (have[i] ?? 0) - (need[i] ?? 0);
    if (difference !== 0) return difference < 0;
  }
  return false;
}
