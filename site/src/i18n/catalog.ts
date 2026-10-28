// Loads the seven TOML files at build time (Vite bundles them as text).

import type { Catalogs } from "./messages.ts";
import { flatten } from "./messages.ts";

const sources = import.meta.glob<string>("./messages/*.toml", {
  query: "?raw",
  import: "default",
  eager: true,
});

const read = (file: string) => {
  const source = sources[`./messages/${file}.toml`];
  if (source === undefined) throw new Error(`Missing locale file messages/${file}.toml`);
  return flatten(source, file);
};

export const CATALOGS: Catalogs = {
  en: read("en"),
  "pt-BR": read("pt-BR"),
  "pt-PT": read("pt-PT"),
  es: read("es"),
  fr: read("fr"),
  it: read("it"),
  de: read("de"),
};
