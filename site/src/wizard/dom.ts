// Small typed lookups for the wizard's markup. A missing element is a bug in the page, so
// these throw instead of returning null.

export function one<T extends HTMLElement = HTMLElement>(root: ParentNode, selector: string): T {
  const element = root.querySelector<T>(selector);
  if (!element) throw new Error(`The wizard's markup has no ${selector}`);
  return element;
}

export function all<T extends HTMLElement = HTMLElement>(root: ParentNode, selector: string): T[] {
  return [...root.querySelectorAll<T>(selector)];
}
