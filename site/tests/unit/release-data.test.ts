import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import {
  assemble,
  compareTags,
  fetchReleaseData,
  type GithubRelease,
  parseSums,
  selectRelease,
} from "../../scripts/data/release.ts";

const fixture = (name: string) =>
  readFileSync(new URL(`../fixtures/${name}`, import.meta.url), "utf8");
const releases = JSON.parse(fixture("github-releases.json")) as GithubRelease[];
const sums = parseSums(fixture("SHA256SUMS"));
const bare = (tag: string, extra: Partial<GithubRelease> = {}): GithubRelease => ({
  tag_name: tag,
  draft: false,
  prerelease: true,
  published_at: "2026-01-01T00:00:00Z",
  html_url: `https://example.test/${tag}`,
  assets: [],
  ...extra,
});

describe("compareTags", () => {
  test("compares numbers, not text", () => {
    expect(compareTags("v0.10.0", "v0.9.0")).toBe(1);
    expect(compareTags("v1.0.0", "v0.99.99")).toBe(1);
    expect(compareTags("v1.2.3", "v1.2.3")).toBe(0);
  });
  test("a prerelease is lower than its release, and prerelease identifiers follow SemVer", () => {
    expect(compareTags("v1.0.0-rc.1", "v1.0.0")).toBe(-1);
    expect(compareTags("v1.0.0-rc.2", "v1.0.0-rc.10")).toBe(-1);
    expect(compareTags("v1.0.0-alpha", "v1.0.0-alpha.1")).toBe(-1);
    expect(compareTags("v1.0.0-beta", "v1.0.0-alpha")).toBe(1);
  });
  test("rejects tags that are not versions", () => {
    expect(() => compareTags("nightly", "v1.0.0")).toThrow("Not a version tag");
  });
});

describe("selectRelease", () => {
  test("picks the highest version, prereleases included, skipping drafts and other tags", () => {
    expect(selectRelease(releases)?.tag_name).toBe("v0.2.0");
  });
  test("0.10 beats 0.9", () => {
    expect(selectRelease([bare("v0.9.0"), bare("v0.10.0"), bare("v0.2.0")])?.tag_name).toBe(
      "v0.10.0",
    );
  });
  test("no releases (or only drafts) means null", () => {
    expect(selectRelease([])).toBeNull();
    expect(selectRelease([bare("v1.0.0", { draft: true })])).toBeNull();
  });
});

describe("parseSums", () => {
  test("reads both the text and the binary marker, lower-casing hashes", () => {
    const hash = "A".repeat(64);
    const parsed = parseSums(`${hash}  one.zip\n${hash} *two.zip\nnot a line\n`);
    expect(parsed.get("one.zip")).toBe("a".repeat(64));
    expect(parsed.get("two.zip")).toBe("a".repeat(64));
    expect(parsed.size).toBe(2);
  });
});

describe("assemble", () => {
  const chosen = () => releases[0] as GithubRelease;

  test("builds every file of the release table with its size and hash", () => {
    const info = assemble(chosen(), sums, "signlocal");
    expect(info.version).toBe("0.2.0");
    expect(info.prerelease).toBe(true);
    expect(Object.keys(info.assets)).toHaveLength(13);
    const file = info.assets["signlocal-0.2.0-windows-x64.zip"];
    expect(file?.size).toBe(6_512_345);
    expect(file?.sha256).toBe(sums.get("signlocal-0.2.0-windows-x64.zip") as string);
    expect(file?.url).toBe(
      "https://github.com/diagnos-tech/signlocal/releases/download/v0.2.0/signlocal-0.2.0-windows-x64.zip",
    );
  });

  test("a missing asset fails and names the file", () => {
    const release = {
      ...chosen(),
      assets: chosen().assets.filter((asset) => asset.name !== "install.ps1"),
    };
    expect(() => assemble(release, sums, "signlocal")).toThrow("no asset install.ps1");
  });

  test("a file missing from SHA256SUMS fails", () => {
    const partial = new Map(sums);
    partial.delete("signlocal-0.2.0-macos-universal.zip");
    expect(() => assemble(chosen(), partial, "signlocal")).toThrow(
      "does not list signlocal-0.2.0-macos-universal.zip",
    );
  });

  test("a digest GitHub computed that differs from SHA256SUMS fails", () => {
    const tampered = {
      ...chosen(),
      assets: chosen().assets.map((asset) =>
        asset.name === "install.sh" ? { ...asset, digest: `sha256:${"0".repeat(64)}` } : asset,
      ),
    };
    expect(() => assemble(tampered, sums, "signlocal")).toThrow("differs from SHA256SUMS");
  });
});

describe("fetchReleaseData", () => {
  const answer = (body: unknown, ok = true, status = 200) =>
    ({ ok, status, json: async () => body, text: async () => String(body) }) as Response;
  /** A fetch that answers with `handler`; Bun's `fetch` type also wants `preconnect`. */
  const fakeFetch = (handler: (url: string, init?: RequestInit) => Response): typeof fetch =>
    Object.assign(
      async (input: string | URL | Request, init?: RequestInit) => handler(String(input), init),
      {
        preconnect: () => undefined,
      },
    );

  test("zero releases is valid and means null", async () => {
    const data = await fetchReleaseData({
      repositoryPath: "o/r",
      slug: "signlocal",
      fetchImpl: fakeFetch(() => answer([])),
      now: () => new Date("2026-11-12T00:00:00Z"),
    });
    expect(data).toEqual({
      source: "github",
      generatedAt: "2026-11-12T00:00:00.000Z",
      release: null,
    });
  });

  test("asks the API with the token and downloads SHA256SUMS of the chosen release", async () => {
    const calls: Array<{ url: string; auth: string | undefined }> = [];
    const fetchImpl = fakeFetch((url, init) => {
      calls.push({
        url,
        auth: (init?.headers as Record<string, string> | undefined)?.Authorization,
      });
      return url.includes("api.github.com") ? answer(releases) : answer(fixture("SHA256SUMS"));
    });
    const data = await fetchReleaseData({
      repositoryPath: "o/r",
      slug: "signlocal",
      token: "t0ken",
      fetchImpl,
    });
    expect(data.release?.tag).toBe("v0.2.0");
    expect(calls[0]).toEqual({
      url: "https://api.github.com/repos/o/r/releases?per_page=30",
      auth: "Bearer t0ken",
    });
    expect(calls[1]?.url).toEndWith("/SHA256SUMS");
  });

  test("an API error rejects", async () => {
    const failing = fakeFetch(() => answer({}, false, 403));
    expect(
      fetchReleaseData({ repositoryPath: "o/r", slug: "signlocal", fetchImpl: failing }),
    ).rejects.toThrow("403");
  });
});
