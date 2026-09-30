/**
 * @websign/sdk/messages — user-facing texts for SDK error codes, in the
 * shipped locales (generated from i18n/*.toml `[site.errors]`). A separate
 * entry point so sites that bring their own texts do not ship these.
 */

import type { ErrorCode } from "./generated/index.js";
import { MESSAGES } from "./messages.gen.js";

/** Locales with texts. */
export type MessageLocale = "en" | "pt-BR" | "pt-PT" | "es" | "fr" | "it" | "de";

const LOCALES: readonly MessageLocale[] = ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"];

/** A title and a sentence for one error. */
export interface ErrorText {
  readonly title: string;
  readonly body: string;
}

/**
 * Texts for `code` in the closest shipped locale to `locale` (BCP 47), with
 * `{installed}`/`{required}` left for the caller to fill; undefined for codes
 * a site never shows (InvalidRequest, PinIncorrect, ClientOutdated).
 */
export function errorText(code: ErrorCode, locale: string): ErrorText | undefined {
  const texts: Partial<Record<ErrorCode, ErrorText>> = MESSAGES[closest(locale)];
  const entry = texts[code];
  return entry && { title: entry.title, body: entry.body };
}

/** `pt` → pt-BR, `es-MX` → es, `PT-pt` → pt-PT, anything unknown → en. */
function closest(locale: string): MessageLocale {
  const tag = locale.replace("_", "-").toLowerCase();
  const exact = LOCALES.find((l) => l.toLowerCase() === tag);
  if (exact) return exact;
  const language = tag.split("-")[0];
  return language === "pt" ? "pt-BR" : (LOCALES.find((l) => l === language) ?? "en");
}
