// Sizes and dates in the visitor's language.

import type { Locale } from "../i18n/locales.ts";

const KB = 1024;
const MB = KB * 1024;

/** "214 KB", "6.2 MB": one decimal for megabytes, with the language's decimal mark. */
export function formatBytes(bytes: number, language: string): string {
  const number = (value: number, digits: number) =>
    new Intl.NumberFormat(language, { maximumFractionDigits: digits }).format(value);
  if (bytes >= MB) return `${number(bytes / MB, 1)} MB`;
  if (bytes >= KB) return `${number(Math.round(bytes / KB), 0)} KB`;
  return `${number(bytes, 0)} B`;
}

/** English dates read day first ("12 November 2026"), like the other languages of the site. */
const intlTag = (code: Locale["code"]) => (code === "en" ? "en-GB" : code);

/** "12 November 2026", "12 de novembro de 2026"; always in UTC so builds agree everywhere. */
export function formatDate(iso: string, locale: Pick<Locale, "code">): string {
  return new Intl.DateTimeFormat(intlTag(locale.code), {
    dateStyle: "long",
    timeZone: "UTC",
  }).format(new Date(iso));
}
