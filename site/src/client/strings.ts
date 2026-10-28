// Reads the texts a page rendered for a client module (components/ui/ClientStrings.astro).

import type { MessageKey } from "../generated/message-keys.ts";

export type Strings<Key extends MessageKey> = Readonly<Record<Key, string>>;

/** The JSON block "<module>-strings"; throws when it is missing or lacks a key. */
export function readStrings<const Keys extends readonly MessageKey[]>(
  module: string,
  keys: Keys,
): Strings<Keys[number]> {
  const block = document.getElementById(`${module}-strings`);
  if (!block?.textContent) throw new Error(`The page has no ${module}-strings block`);
  const strings = JSON.parse(block.textContent) as Record<string, string>;
  for (const key of keys) {
    if (strings[key] === undefined) throw new Error(`${module}-strings has no ${key}`);
  }
  return strings as Strings<Keys[number]>;
}

/** Fills {placeholders}; the pages' own variables, so a missing one is a bug. */
export function fill(
  template: string,
  variables: Readonly<Record<string, string | number>>,
): string {
  return template.replace(/\{([a-z_]+)\}/g, (_all, name: string) =>
    String(variables[name] ?? `{${name}}`),
  );
}
