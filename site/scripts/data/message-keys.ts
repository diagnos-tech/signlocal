// en.toml → src/generated/message-keys.ts: the union type of every message key, so that
// `t("section.key")` is checked by `astro check` and a misspelled key never reaches a page.

import { readFileSync } from "node:fs";
import { flatten } from "../../src/i18n/messages.ts";

export function messageKeysModule(englishToml: string): string {
  const keys = Object.keys(flatten(englishToml, "en.toml"));
  const union = keys.map((key) => `  | "${key}"`).join("\n");
  return `// Written by scripts/data/message-keys.ts from en.toml. Do not edit.\n\nexport type MessageKey =\n${union};\n`;
}

export function readMessageKeys(site: string): string {
  return messageKeysModule(readFileSync(`${site}/src/i18n/messages/en.toml`, "utf8"));
}
