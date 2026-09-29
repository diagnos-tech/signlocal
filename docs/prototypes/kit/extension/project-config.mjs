// Reads the identifiers the extension and its tests need from the
// repository's project.toml, so no ID or name is written down twice.
//
// project.toml only uses `[section]` headers and `key = "string"` pairs, so a
// tiny parser is enough; anything else is ignored.

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));

/** Absolute path of the repository's project.toml. */
export const PROJECT_TOML = resolve(here, "../../../../project.toml");

/** Parses `[section]` / `key = "value"` TOML into `{ section: { key: value } }`. */
export function parseProjectToml(text) {
  const tables = {};
  let table = null;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line === "" || line.startsWith("#")) continue;
    const header = /^\[([A-Za-z0-9_.-]+)\]$/.exec(line);
    if (header) {
      table = tables[header[1]] = {};
      continue;
    }
    const pair = /^([A-Za-z0-9_-]+)\s*=\s*"((?:[^"\\]|\\.)*)"\s*(?:#.*)?$/.exec(line);
    if (pair && table) {
      table[pair[1]] = pair[2].replace(/\\(["\\])/g, "$1");
    }
  }
  return tables;
}

/** The values the extension needs, failing loudly when one is missing. */
export function readProject(path = PROJECT_TOML) {
  const toml = parseProjectToml(readFileSync(path, "utf8"));
  const need = (section, key) => {
    const value = toml[section]?.[key];
    if (typeof value !== "string" || value === "") {
      throw new Error(`project.toml: [${section}].${key} is missing or empty`);
    }
    return value;
  };
  return {
    productName: need("product", "name"),
    slug: need("product", "slug"),
    nativeHost: need("ids", "native_host"),
    firefoxId: need("extension", "firefox_id"),
    devId: need("extension", "dev_id"),
    devKey: need("extension", "dev_key"),
  };
}
