// project.toml and the SDK's package files → src/generated/project.json.
// Fails, naming the file, when a key is missing or the shape changed.

import { readFileSync } from "node:fs";
import { parse } from "smol-toml";
import type { ProjectData } from "../../src/lib/data-types.ts";

const text = (value: unknown, where: string): string => {
  if (typeof value !== "string") throw new Error(`project.toml: ${where} is missing or not text`);
  return value;
};

export function readProject(repo: string): ProjectData {
  const toml = parse(readFileSync(`${repo}/project.toml`, "utf8")) as Record<
    string,
    Record<string, unknown>
  >;
  const product = toml.product ?? {};
  const ids = toml.ids ?? {};
  const extension = toml.extension ?? {};
  const repository = text(product.repository, "product.repository");
  const path = new URL(repository).pathname.replace(/^\/|\/$/g, "");
  const homepage = text(product.homepage, "product.homepage");
  if (!homepage.endsWith("/")) throw new Error("project.toml: product.homepage must end with /");
  return {
    name: text(product.name, "product.name"),
    slug: text(product.slug, "product.slug"),
    tagline: text(product.tagline, "product.tagline"),
    repository,
    repositoryPath: path,
    homepage,
    urlScheme: text(ids.url_scheme, "ids.url_scheme"),
    minAppVersion: text(extension.min_app_version, "extension.min_app_version"),
    extension: {
      chromeWebStoreId: text(extension.chrome_web_store_id, "extension.chrome_web_store_id"),
      edgeAddonsId: text(extension.edge_addons_id, "extension.edge_addons_id"),
      firefoxAmoSlug: text(extension.firefox_amo_slug, "extension.firefox_amo_slug"),
    },
    sdk: readSdk(repo),
  };
}

/** The SDK's version and the gzip budget its size check enforces. */
export function readSdk(repo: string): ProjectData["sdk"] {
  const manifest = JSON.parse(readFileSync(`${repo}/sdk/package.json`, "utf8")) as {
    version?: string;
  };
  if (!manifest.version) throw new Error("sdk/package.json: version is missing");
  const size = readFileSync(`${repo}/sdk/scripts/size.ts`, "utf8");
  const budget = /BUDGET_BYTES\s*=\s*(\d+)\s*\*\s*1024/.exec(size);
  if (!budget?.[1]) throw new Error("sdk/scripts/size.ts: BUDGET_BYTES = <n> * 1024 not found");
  return { version: manifest.version, budgetKb: Number(budget[1]) };
}
