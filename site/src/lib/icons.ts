// The inline icons: Phosphor (MIT) for the interface and the operating systems.

import { type IconSize, prepareSvg } from "./svg.ts";

const files = import.meta.glob<string>("../icons/*/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

const byName = new Map<string, string>();
for (const [path, raw] of Object.entries(files)) {
  const name = /\/([^/]+)\.svg$/.exec(path)?.[1];
  if (name) byName.set(name, raw);
}

/** The inline SVG of an icon; an unknown name fails the build. */
export function icon(name: string, size: IconSize): string {
  const raw = byName.get(name);
  if (raw === undefined) throw new Error(`Unknown icon "${name}": add its SVG to src/icons/`);
  return prepareSvg(raw, name, size);
}
