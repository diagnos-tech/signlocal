// Prepares an icon's SVG text for inlining: removes fixed dimensions, hides it from assistive
// technology and adds the size class. An icon without a viewBox cannot scale, so it fails.

export type IconSize = 16 | 20 | 24 | 32 | 48;

export function prepareSvg(raw: string, name: string, size: IconSize): string {
  const open = /<svg\b[^>]*>/.exec(raw);
  if (!open || !/\bviewBox="[^"]+"/.test(open[0])) throw new Error(`Icon "${name}" has no viewBox`);
  const tag = open[0]
    .replace(/\s(?:width|height)="[^"]*"/g, "")
    .replace(/>$/, ` aria-hidden="true" focusable="false" class="icon icon--${size}">`);
  return raw.replace(open[0], tag).trim();
}
