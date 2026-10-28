// What the product itself says it needs → src/generated/support.json:
// the minimum browser versions (extension manifest), the minimum macOS (Info.plist) and the
// Linux distributions the end-to-end suite runs on (.github/workflows/e2e.yml).

import { readFileSync } from "node:fs";
import type { SupportData } from "../../src/lib/data-types.ts";

function match(file: string, pattern: RegExp, what: string): string {
  const found = pattern.exec(file);
  if (!found?.[1]) throw new Error(`Could not find ${what}`);
  return found[1];
}

const DISTRIBUTIONS: Readonly<Record<string, string>> = {
  "ubuntu-22.04": "Ubuntu 22.04",
  "ubuntu-24.04": "Ubuntu 24.04",
  "debian-12": "Debian 12",
  "fedora-42": "Fedora 42",
  "rockylinux-9": "Rocky Linux 9",
  archlinux: "Arch Linux",
};

/** Display names of the distributions in e2e.yml: Ubuntu runners, then the container matrix. */
export function linuxTested(workflow: string): string[] {
  const parsed = Bun.YAML.parse(workflow) as {
    jobs?: Record<
      string,
      { strategy?: { matrix?: { os?: unknown; include?: { os?: string }[] } } }
    >;
  };
  const ids: string[] = [];
  for (const job of Object.values(parsed.jobs ?? {})) {
    const matrix = job.strategy?.matrix;
    if (typeof matrix?.os === "string") ids.push(...(matrix.os.match(/ubuntu-\d+\.\d+/g) ?? []));
    for (const entry of matrix?.include ?? []) if (entry.os) ids.push(entry.os);
  }
  const unique = [...new Set(ids)];
  if (unique.length === 0)
    throw new Error(".github/workflows/e2e.yml: no Linux distributions found");
  return unique.map((id) => {
    const name = DISTRIBUTIONS[id];
    if (!name)
      throw new Error(`e2e.yml names "${id}": add it to DISTRIBUTIONS in scripts/data/support.ts`);
    return name;
  });
}

export function readSupport(repo: string): SupportData {
  const manifest = readFileSync(`${repo}/extension/build/manifest.ts`, "utf8");
  const plist = readFileSync(`${repo}/packaging/macos/Info.plist.in`, "utf8");
  const workflow = readFileSync(`${repo}/.github/workflows/e2e.yml`, "utf8");
  return {
    chromium: match(
      manifest,
      /minimum_chrome_version:\s*"(\d+)"/,
      "minimum_chrome_version in extension/build/manifest.ts",
    ),
    firefox: match(
      manifest,
      /strict_min_version:\s*"(\d+)/,
      "strict_min_version in extension/build/manifest.ts",
    ),
    macos: match(
      plist,
      /LSMinimumSystemVersion<\/key>\s*<string>(\d+)/,
      "LSMinimumSystemVersion in Info.plist.in",
    ),
    linuxTested: linuxTested(workflow),
  };
}
