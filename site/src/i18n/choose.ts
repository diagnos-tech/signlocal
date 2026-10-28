// Picks the language for a visitor: ?lang, then the saved choice, then the browser's languages.
// Pure, so the boot script and the tests share it. Matching rules: exact tag, `pt` means
// Brazilian Portuguese unless the tag says pt-PT, then the base language ("de-AT" → de).

import { LOCALE_CODES, type LocaleCode } from "./codes.ts";

const CODES = new Map<string, LocaleCode>(LOCALE_CODES.map((code) => [code.toLowerCase(), code]));

/** The locale a language tag stands for, or undefined when the site has none for it. */
export function matchTag(tag: string | null | undefined): LocaleCode | undefined {
  if (!tag) return undefined;
  const normal = tag.replace("_", "-").toLowerCase();
  const exact = CODES.get(normal);
  if (exact) return exact;
  const base = normal.split("-")[0] ?? "";
  if (base === "pt") return "pt-BR";
  return CODES.get(base);
}

export interface Preferences {
  /** The value of ?lang=, if any. */
  readonly query?: string | null;
  /** The choice remembered in localStorage, if any. */
  readonly saved?: string | null;
  /** navigator.languages (or [navigator.language]). */
  readonly languages?: readonly string[];
}

/** The visitor's locale; English when nothing matches. */
export function choose({ query, saved, languages = [] }: Preferences): LocaleCode {
  const explicit = matchTag(query) ?? matchTag(saved);
  if (explicit) return explicit;
  for (const tag of languages) {
    const found = matchTag(tag);
    if (found) return found;
  }
  return "en";
}
