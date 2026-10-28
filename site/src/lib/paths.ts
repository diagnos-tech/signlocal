// `getStaticPaths` of every [lang] route: one page per language other than English.

import { PREFIXED_LOCALES } from "../i18n/locales.ts";

export function languagePaths() {
  return PREFIXED_LOCALES.map((locale) => ({
    params: { lang: locale.segment },
    props: { locale: locale.code },
  }));
}
