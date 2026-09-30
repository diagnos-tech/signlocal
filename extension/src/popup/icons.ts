/**
 * The popup's inline SVG icons (docs/ux.md §9). Paths are constants written
 * here, never built from data, so injecting them as markup is safe.
 */

export type IconName =
  | "spinner"
  | "check-circle"
  | "download-simple"
  | "warning"
  | "x-circle"
  | "info";

const SHAPES: Readonly<Record<IconName, string>> = {
  spinner: '<path d="M21 12a9 9 0 1 1-6.2-8.56"/>',
  "check-circle": '<circle cx="12" cy="12" r="9"/><path d="m8 12.5 2.8 2.8L16 9.5"/>',
  "download-simple": '<path d="M12 4v11m-5-4.5 5 5 5-5M5 20h14"/>',
  warning: '<path d="M12 4 2.8 20h18.4z"/><path d="M12 10v4.5m0 2.5v.01"/>',
  "x-circle": '<circle cx="12" cy="12" r="9"/><path d="m9 9 6 6m0-6-6 6"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5m0-8v.01"/>',
};

/** Markup of a decorative `size`-px icon; screen readers skip it. */
export function icon(name: IconName, size: number): string {
  return (
    `<svg class="i-${name}" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" ` +
    `stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" ` +
    `aria-hidden="true" focusable="false">${SHAPES[name]}</svg>`
  );
}
