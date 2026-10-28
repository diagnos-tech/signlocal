// The seven languages of the site. English lives at the root; the others under a folder named
// after the language code in lower case (/pt-br/, /es/ …).

import { folderOf, LOCALE_CODES, type LocaleCode } from "./codes.ts";

export type { LocaleCode };

export interface Locale {
  readonly code: LocaleCode;
  /** The folder's name, which is the `[lang]` route parameter; "" for English. */
  readonly segment: string;
  /** The folder with its leading slash, "" for English. */
  readonly folder: string;
  /** The language's own name, as the language menu shows it. */
  readonly endonym: string;
  /** Two letters for the narrow header. */
  readonly short: string;
  /** Open Graph locale (underscore form). */
  readonly ogLocale: string;
}

const DETAILS: Readonly<Record<LocaleCode, Pick<Locale, "endonym" | "short" | "ogLocale">>> = {
  en: { endonym: "English", short: "EN", ogLocale: "en_US" },
  "pt-BR": { endonym: "Português (Brasil)", short: "PT", ogLocale: "pt_BR" },
  "pt-PT": { endonym: "Português (Portugal)", short: "PT", ogLocale: "pt_PT" },
  es: { endonym: "Español", short: "ES", ogLocale: "es_ES" },
  fr: { endonym: "Français", short: "FR", ogLocale: "fr_FR" },
  it: { endonym: "Italiano", short: "IT", ogLocale: "it_IT" },
  de: { endonym: "Deutsch", short: "DE", ogLocale: "de_DE" },
};

export const LOCALES: readonly Locale[] = LOCALE_CODES.map((code) => ({
  code,
  folder: folderOf(code),
  segment: folderOf(code).slice(1),
  ...DETAILS[code],
}));

/** The locales that live under a folder (everything but English). */
export const PREFIXED_LOCALES = LOCALES.filter((locale) => locale.segment !== "");

const BY_CODE = new Map<string, Locale>(LOCALES.map((locale) => [locale.code, locale]));

export function localeOf(code: string): Locale {
  const locale = BY_CODE.get(code);
  if (!locale) throw new Error(`Unknown locale "${code}"`);
  return locale;
}

/** The locale a folder name stands for ("pt-br" → pt-BR). */
export function localeOfSegment(segment: string): Locale {
  const locale = PREFIXED_LOCALES.find((candidate) => candidate.segment === segment);
  if (!locale) throw new Error(`No locale for the folder "${segment}"`);
  return locale;
}
