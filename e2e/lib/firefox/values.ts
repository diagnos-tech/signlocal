/**
 * JSON-like values to and from WebDriver BiDi's typed form (`LocalValue`
 * out, `RemoteValue` back). The fixture page answers in plain JSON, so
 * strings, numbers, booleans, null, arrays and plain objects are all the
 * suite ever passes.
 */

/** A BiDi-typed value (both directions share this shape for JSON data). */
export type Typed =
  | { readonly type: "undefined" | "null" }
  | { readonly type: "string"; readonly value: string }
  | { readonly type: "number"; readonly value: number | string }
  | { readonly type: "boolean"; readonly value: boolean }
  | { readonly type: "bigint"; readonly value: string }
  | { readonly type: "array"; readonly value?: readonly Typed[] }
  | { readonly type: "object"; readonly value?: ReadonlyArray<readonly [string | Typed, Typed]> }
  | { readonly type: string; readonly value?: unknown };

/** `value` as a BiDi `LocalValue`. */
export function toTyped(value: unknown): Typed {
  if (value === undefined) return { type: "undefined" };
  if (value === null) return { type: "null" };
  switch (typeof value) {
    case "string":
      return { type: "string", value };
    case "boolean":
      return { type: "boolean", value };
    case "number":
      return { type: "number", value: Number.isFinite(value) ? value : String(value) };
    case "bigint":
      return { type: "bigint", value: value.toString() };
    case "object":
      if (Array.isArray(value)) return { type: "array", value: value.map(toTyped) };
      return {
        type: "object",
        value: Object.entries(value).map(([key, item]) => [key, toTyped(item)] as const),
      };
    default:
      throw new TypeError(`cannot pass a ${typeof value} to the page`);
  }
}

/** A BiDi `RemoteValue` as a plain value; handles (nodes, windows…) are refused. */
export function fromTyped(typed: Typed): unknown {
  switch (typed.type) {
    case "undefined":
      return undefined;
    case "null":
      return null;
    case "string":
    case "boolean":
      return typed.value;
    case "number":
      return typeof typed.value === "number" ? typed.value : Number(typed.value);
    case "bigint":
      return BigInt(typed.value as string);
    case "array":
      return ((typed.value ?? []) as readonly Typed[]).map(fromTyped);
    case "object":
      return Object.fromEntries(
        ((typed.value ?? []) as ReadonlyArray<readonly [string | Typed, Typed]>).map(
          ([key, item]) => [
            typeof key === "string" ? key : String(fromTyped(key)),
            fromTyped(item),
          ],
        ),
      );
    default:
      throw new TypeError(`the page returned a ${typed.type}, not JSON data`);
  }
}
