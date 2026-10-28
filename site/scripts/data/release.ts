// The newest release of the repository → src/generated/release.json.
//
// Version, date, file sizes and SHA-256 hashes come from the GitHub Releases API and the
// release's own SHA256SUMS, at build time: nothing about a release is committed, and a build
// that cannot prove its data is right fails instead of publishing it (see run.ts).

import type { ReleaseAsset, ReleaseData, ReleaseInfo } from "../../src/lib/data-types.ts";
import { requiredFiles } from "../../src/platform/artifacts.ts";

export interface GithubAsset {
  readonly name: string;
  readonly size: number;
  readonly browser_download_url: string;
  /** "sha256:<hex>" on assets GitHub has hashed. */
  readonly digest?: string | null;
}

export interface GithubRelease {
  readonly tag_name: string;
  readonly draft: boolean;
  readonly prerelease: boolean;
  readonly published_at: string | null;
  readonly html_url: string;
  readonly assets: readonly GithubAsset[];
}

const TAG = /^v(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/;

/** SemVer 2.0.0 precedence: -1, 0 or 1. Tags must match TAG. */
export function compareTags(a: string, b: string): number {
  const left = TAG.exec(a);
  const right = TAG.exec(b);
  if (!left || !right) throw new Error(`Not a version tag: ${!left ? a : b}`);
  for (const index of [1, 2, 3] as const) {
    const difference = Number(left[index]) - Number(right[index]);
    if (difference !== 0) return Math.sign(difference);
  }
  return comparePrerelease(left[4], right[4]);
}

function comparePrerelease(a: string | undefined, b: string | undefined): number {
  if (a === undefined || b === undefined) return a === b ? 0 : a === undefined ? 1 : -1;
  const left = a.split(".");
  const right = b.split(".");
  for (let index = 0; index < Math.max(left.length, right.length); index += 1) {
    const x = left[index];
    const y = right[index];
    if (x === undefined) return -1;
    if (y === undefined) return 1;
    const numeric = /^\d+$/.test(x) && /^\d+$/.test(y);
    if (numeric && Number(x) !== Number(y)) return Math.sign(Number(x) - Number(y));
    if (!numeric && x !== y) return x < y ? -1 : 1;
  }
  return 0;
}

/** The non-draft release with the highest SemVer tag, prereleases included; null when none. */
export function selectRelease(releases: readonly GithubRelease[]): GithubRelease | null {
  const candidates = releases.filter((release) => !release.draft && TAG.test(release.tag_name));
  candidates.sort((a, b) => compareTags(b.tag_name, a.tag_name));
  return candidates[0] ?? null;
}

/** Lines of SHA256SUMS: "<64 hex>  <name>" or "<64 hex> *<name>". */
export function parseSums(text: string): Map<string, string> {
  const sums = new Map<string, string>();
  for (const line of text.split(/\r?\n/)) {
    const found = /^([0-9a-fA-F]{64}) [ *](.+)$/.exec(line.trim());
    if (found?.[1] && found[2]) sums.set(found[2], found[1].toLowerCase());
  }
  return sums;
}

/** Every file of the release table, with its size and hash; throws on any inconsistency. */
export function assemble(
  release: GithubRelease,
  sums: Map<string, string>,
  slug: string,
): ReleaseInfo {
  const version = release.tag_name.slice(1);
  const byName = new Map(release.assets.map((asset) => [asset.name, asset]));
  const assets: Record<string, ReleaseAsset> = {};
  for (const name of requiredFiles(slug, version)) {
    const asset = byName.get(name);
    if (!asset) throw new Error(`${release.tag_name}: the release has no asset ${name}`);
    const sha256 = sums.get(name);
    if (!sha256) throw new Error(`${release.tag_name}: SHA256SUMS does not list ${name}`);
    const digest = asset.digest?.replace(/^sha256:/, "").toLowerCase();
    if (digest && digest !== sha256) {
      throw new Error(`${release.tag_name}: GitHub's digest of ${name} differs from SHA256SUMS`);
    }
    assets[name] = { name, url: asset.browser_download_url, size: asset.size, sha256 };
  }
  return {
    version,
    tag: release.tag_name,
    prerelease: release.prerelease,
    publishedAt: release.published_at ?? new Date(0).toISOString(),
    url: release.html_url,
    assets,
  };
}

export interface FetchOptions {
  readonly repositoryPath: string;
  readonly slug: string;
  readonly token?: string | undefined;
  readonly fetchImpl?: typeof fetch;
  readonly now?: () => Date;
}

/** The latest release from GitHub. Rejects on any network, API or consistency error. */
export async function fetchReleaseData(options: FetchOptions): Promise<ReleaseData> {
  const get = options.fetchImpl ?? fetch;
  const headers: Record<string, string> = {
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
  };
  if (options.token) headers.Authorization = `Bearer ${options.token}`;
  const url = `https://api.github.com/repos/${options.repositoryPath}/releases?per_page=30`;
  const response = await get(url, { headers });
  if (!response.ok) throw new Error(`GET ${url} answered ${response.status}`);
  const chosen = selectRelease((await response.json()) as GithubRelease[]);
  const generatedAt = (options.now?.() ?? new Date()).toISOString();
  if (!chosen) return { source: "github", generatedAt, release: null };
  const sumsAsset = chosen.assets.find((asset) => asset.name === "SHA256SUMS");
  if (!sumsAsset) throw new Error(`${chosen.tag_name}: the release has no SHA256SUMS`);
  const sumsResponse = await get(sumsAsset.browser_download_url);
  if (!sumsResponse.ok) throw new Error(`Downloading SHA256SUMS answered ${sumsResponse.status}`);
  const release = assemble(chosen, parseSums(await sumsResponse.text()), options.slug);
  return { source: "github", generatedAt, release };
}
