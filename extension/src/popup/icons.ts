/**
 * The popup's inline SVG icons (24-unit grid, 2-unit round strokes, like the
 * app's Phosphor set in docs/ux.md §12). Paths are constants written here,
 * never built from data, so injecting them as markup is safe.
 */

export type IconName =
  | "spinner"
  | "brand"
  | "check-circle"
  | "download"
  | "warning"
  | "x-circle"
  | "info"
  | "pulse"
  | "retry"
  | "external";

const SHAPES: Readonly<Record<IconName, string>> = {
  spinner: '<path d="M21 12a9 9 0 1 1-6.2-8.56"/>',
  // The site's mark (site/assets/icon.svg): a pen and its signature stroke.
  brand: '<path d="M3 20c4 0 5-9 8-9s1 6 4 6 3-4 6-4"/><path d="m4 12 4-4 3 3-4 4z"/>',
  "check-circle": '<circle cx="12" cy="12" r="9"/><path d="m8 12.5 2.8 2.8L16 9.5"/>',
  download: '<path d="M12 4v11m-5-4.5 5 5 5-5M5 20h14"/>',
  warning: '<path d="M12 4 2.8 20h18.4z"/><path d="M12 10v4.5m0 2.5v.01"/>',
  "x-circle": '<circle cx="12" cy="12" r="9"/><path d="m9 9 6 6m0-6-6 6"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5m0-8v.01"/>',
  pulse: '<path d="M3 12h4l3-7 4 14 3-7h4"/>',
  retry: '<path d="M20 4v5h-5"/><path d="M19.5 9A8 8 0 1 0 20 14"/>',
  external: '<path d="M14 4h6v6m0-6-9 9"/><path d="M18 14v5H5V6h5"/>',
};

/** Markup of a decorative `size`-px icon; screen readers skip it. */
export function icon(name: IconName, size: number): string {
  return (
    `<svg class="i-${name}" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" ` +
    `stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" ` +
    `aria-hidden="true" focusable="false">${SHAPES[name]}</svg>`
  );
}
