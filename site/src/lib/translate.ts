// What every component needs to say something: the translator and the rich-text renderer of
// its language, wired to the site's links.

import type { MessageKey } from "../generated/message-keys.ts";
import { CATALOGS } from "../i18n/catalog.ts";
import type { LocaleCode } from "../i18n/codes.ts";
import { type Translator, translator } from "../i18n/messages.ts";
import type { Variables } from "../i18n/rich.ts";
import { linkResolver } from "./routes.ts";
import { PROJECT } from "./site.ts";

export interface Text {
  /** Plain text, variables filled. */
  readonly t: Translator;
  /** HTML for `set:html`: markers converted, links localized. */
  readonly html: (key: MessageKey, variables?: Variables) => string;
}

export function text(locale: LocaleCode): Text {
  const t = translator(CATALOGS, locale);
  const resolve = linkResolver(locale, PROJECT);
  return { t, html: (key, variables) => t.html(key, resolve, variables) };
}
