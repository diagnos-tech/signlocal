/**
 * @websign/sdk/messages — texts for people, per error code, in the shipped
 * locales (generated from i18n/*.toml `[site.errors]`). A separate entry
 * point so sites that bring their own texts do not ship these.
 *
 * @packageDocumentation
 * @module @websign/sdk/messages
 */

import type { ErrorCode, ErrorDetails } from "./generated/index.js";
import { MESSAGES } from "./messages.gen.js";

/** Locales with texts. */
export type MessageLocale = "en" | "pt-BR" | "pt-PT" | "es" | "fr" | "it" | "de";

const LOCALES: readonly MessageLocale[] = ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"];

/**
 * A title and a sentence with the next step, ready to show to the person.
 *
 * @example
 * const { title, body } = errorText("NoCertificates", "en") ?? fallback;
 */
export interface ErrorText {
  readonly title: string;
  readonly body: string;
}

/** A `WebSignError`, or anything with its `code` (and `details`). */
export interface ErrorLike {
  readonly code: ErrorCode;
  readonly details?: ErrorDetails | undefined;
}

/**
 * The texts for an error in the closest shipped locale to `locale` (BCP 47,
 * default: the browser's language, else English). Given a `WebSignError`,
 * `{installed}` and `{required}` are filled from its `details`.
 *
 * Returns `undefined` for codes a site never shows people (`InvalidRequest`,
 * `PinIncorrect`, `ClientOutdated`: bugs or internal) and for `undefined`, so
 * `errorText(status.problem)` just works.
 *
 * @example
 * import { errorText } from "@websign/sdk/messages";
 *
 * catch (error) {
 *   if (!isWebSignError(error) || error.code === "UserCancelled") return;
 *   const text = errorText(error, document.documentElement.lang);
 *   if (text) showBanner(text.title, text.body);
 * }
 */
export function errorText(
  error: ErrorCode | ErrorLike | undefined,
  locale: string = (globalThis as { navigator?: Navigator }).navigator?.language ?? "en",
): ErrorText | undefined {
  if (error === undefined) return undefined;
  const { code, details } = typeof error === "string" ? { code: error, details: undefined } : error;
  const entry: ErrorText | undefined = MESSAGES[closest(locale)][code];
  if (entry === undefined) return undefined;
  const fill = (text: string) =>
    text.replace(
      /\{(installed|required)\}/g,
      (all, key: "installed" | "required") => details?.[key] ?? all,
    );
  return { title: fill(entry.title), body: fill(entry.body) };
}

/** `pt` → pt-BR, `es-MX` → es, `PT-pt` → pt-PT, anything unknown → en. */
function closest(locale: string): MessageLocale {
  const tag = locale.replace("_", "-").toLowerCase();
  const exact = LOCALES.find((l) => l.toLowerCase() === tag);
  if (exact) return exact;
  const language = tag.split("-")[0];
  return language === "pt" ? "pt-BR" : (LOCALES.find((l) => l === language) ?? "en");
}
