/** Popup text lookup through `browser.i18n` (catalogs generated from i18n/*.toml). */

import { browser } from "wxt/browser";

/**
 * Localized text for `key`, with `values` filling its `{name}` placeholders.
 *
 * `cargo xtask gen` numbers a message's placeholders in the alphabetical
 * order of their names (translations reorder words), so the values are
 * passed by name and sorted here: a positional call cannot swap them.
 * Falls back to the key so a missing string is visible, not blank.
 */
export function t(key: string, values: Readonly<Record<string, string>> = {}): string {
  const substitutions = Object.keys(values)
    .sort()
    .map((name) => values[name] ?? "");
  return browser.i18n.getMessage(key as never, substitutions) || key;
}
