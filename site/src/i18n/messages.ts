// Message catalogs: flat "section.key" → text maps parsed from the TOML files, and the lookup
// that pages use. A key that does not exist is a build error, never a blank on the page.

import { parse } from "smol-toml";
import type { MessageKey } from "../generated/message-keys.ts";
import type { LocaleCode } from "./locales.ts";
import { type LinkResolver, rich, substitute, type Variables } from "./rich.ts";

export type Catalog = Readonly<Record<string, string>>;
export type Catalogs = Readonly<Record<LocaleCode, Catalog>>;

/** Flattens `[section] key = "text"` into `"section.key": "text"`; rejects plural tables. */
export function flatten(source: string, file = "messages"): Catalog {
  const flat: Record<string, string> = {};
  const walk = (node: Record<string, unknown>, prefix: string) => {
    for (const [name, value] of Object.entries(node)) {
      const key = prefix ? `${prefix}.${name}` : name;
      if (typeof value === "string") flat[key] = value;
      else if (value && typeof value === "object") walk(value as Record<string, unknown>, key);
      else throw new Error(`${file}: "${key}" is not text`);
    }
  };
  walk(parse(source), "");
  return flat;
}

/** The placeholders a message uses, in order of first appearance. */
export function placeholders(text: string): string[] {
  return [...new Set([...text.matchAll(/\{([a-z_]+)\}/g)].map((match) => match[1] ?? ""))];
}

export interface Translator {
  /** Plain text with {variables} filled. */
  (key: MessageKey, variables?: Variables): string;
  /** The message as written, {placeholders} included (for client code that fills them). */
  raw(key: MessageKey): string;
  /** HTML: markers converted, links localized through `resolve`. */
  html(key: MessageKey, resolve: LinkResolver, variables?: Variables): string;
}

export function translator(catalogs: Catalogs, locale: LocaleCode): Translator {
  const catalog = catalogs[locale];
  const text = (key: MessageKey): string => {
    const found = catalog[key];
    if (found === undefined) throw new Error(`No message "${key}" in ${locale}`);
    return found;
  };
  const translate = ((key: MessageKey, variables: Variables = {}) =>
    substitute(text(key), variables)) as Translator;
  translate.raw = text;
  translate.html = (key, resolve, variables = {}) => rich(text(key), resolve, variables);
  return translate;
}
