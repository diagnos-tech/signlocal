// The language codes of the site, in a tiny module of their own: the boot script needs them
// and must stay under a kilobyte, so it cannot import the full locale table.
// A language's folder is its code in lower case ("pt-BR" → "pt-br"); English has none.

export type LocaleCode = "en" | "pt-BR" | "pt-PT" | "es" | "fr" | "it" | "de";

export const LOCALE_CODES: readonly LocaleCode[] = ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"];

export const folderOf = (code: LocaleCode): string =>
  code === "en" ? "" : `/${code.toLowerCase()}`;
