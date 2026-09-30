/**
 * @websign/sdk/messages — user-facing texts for SDK error codes, in the
 * shipped locales (generated from i18n/*.toml `[site.errors]`). A separate
 * entry point so sites that bring their own texts do not ship these.
 */

import type { ErrorCode } from "./generated";

/** Locales with texts. */
export type MessageLocale = "en" | "pt-BR" | "pt-PT" | "es" | "fr" | "it" | "de";

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
  void code;
  void locale;
  throw new Error("unimplemented: SPEC.md §10");
}
