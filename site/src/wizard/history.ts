// What the address bar says about a state. Only what the person (or the link) confirmed is
// written: a value that detection merely preselected must not come back as "chosen" after a
// reload, or the wizard would skip the step that asks about it.

import type { Choices, Step } from "../platform/types.ts";
import type { Field, State } from "./state.ts";
import { serializeUrl } from "./url.ts";

export function confirmed(state: State): Choices {
  const choices: Record<string, string> = {};
  for (const [field, value] of Object.entries(state.choices) as [Field, string][]) {
    if (state.provided.has(field) || field === "device") choices[field] = value;
  }
  return choices as Choices;
}

/** The query string (with "?", or "" when there is nothing to say) of a state at a step. */
export function queryFor(state: State, step: Step = state.step): string {
  const query = serializeUrl(confirmed(state), step);
  return query ? `?${query}` : "";
}
