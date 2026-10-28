// Conditions, not templates: the result step is rendered at build time with every variant
// (each system, processor, distribution, browser, certificate), and an element says when it
// applies with data attributes. JavaScript only shows and hides.
//
//   data-os="windows macos"      shown for any of the listed values
//   data-browser="!safari"       "!" in front of the list: shown for none of the listed values
//
// Attributes: data-os, data-arch, data-distro, data-browser, data-cert, data-channel
// ("store" or "dev": where the browser gets the extension) and data-release ("yes" or "no").

export const CONDITION_KEYS = [
  "os",
  "arch",
  "distro",
  "browser",
  "cert",
  "channel",
  "release",
] as const;
export type ConditionKey = (typeof CONDITION_KEYS)[number];

/** The state of the choices, one value per key, undefined when not chosen. */
export type Facts = Readonly<Partial<Record<ConditionKey, string>>>;
/** The condition attributes of an element, by key without the "data-" prefix. */
export type Conditions = Readonly<Partial<Record<ConditionKey, string | null | undefined>>>;

export function matches(conditions: Conditions, facts: Facts): boolean {
  for (const key of CONDITION_KEYS) {
    const wanted = conditions[key];
    if (wanted === undefined || wanted === null) continue;
    const negated = wanted.startsWith("!");
    const values = (negated ? wanted.slice(1) : wanted).split(/\s+/).filter(Boolean);
    const actual = facts[key];
    const listed = actual !== undefined && values.includes(actual);
    if (negated ? listed : !listed) return false;
  }
  return true;
}

const SELECTOR = CONDITION_KEYS.map((key) => `[data-${key}]`).join(",");

/** Shows or hides every conditional element under `root`. */
export function apply(root: ParentNode, facts: Facts): void {
  for (const element of root.querySelectorAll<HTMLElement>(SELECTOR)) {
    const conditions: Partial<Record<ConditionKey, string | undefined>> = {};
    for (const key of CONDITION_KEYS) {
      const value = element.getAttribute(`data-${key}`);
      if (value !== null) conditions[key] = value;
    }
    element.hidden = !matches(conditions, facts);
  }
}
