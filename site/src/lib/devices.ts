// The tokens and cards signlocal knows, from devices.json at the repository root (CC0): the
// install page points to the driver each one needs. A model missing here still works once its
// driver is installed; this is a convenience, never a list of approved models.

import raw from "../../../devices.json";
import type { DeviceKind } from "../wizard/url.ts";

export interface Device {
  readonly id: string;
  readonly name: string;
  readonly kind: DeviceKind;
  readonly driver?: {
    readonly name: string;
    readonly download?: Readonly<Partial<Record<"windows" | "macos" | "linux", string>>>;
  };
  readonly macos?: { readonly cryptotokenkit: boolean };
}

const all = (raw as { devices: readonly Device[] }).devices;
const byName = (a: Device, b: Device) => a.name.localeCompare(b.name, "en");

/** Tokens and cards, sorted by name, for the install page's pickers. */
export function devicesOfKind(kind: "token" | "card"): readonly Device[] {
  return all.filter((device) => device.kind === kind).sort(byName);
}

/** Every device of every kind, for the compatibility page. */
export const ALL_DEVICES: readonly Device[] = [...all].sort(byName);

/** id → kind of the devices a person can pick (readers are not certificate holders). */
export function pickableKinds(): Record<string, "token" | "card"> {
  return Object.fromEntries(
    all.filter((d) => d.kind !== "reader").map((d) => [d.id, d.kind as "token" | "card"]),
  );
}
