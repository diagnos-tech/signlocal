/**
 * Where to send a person who lacks the extension: the store of the current
 * browser (Chrome Web Store, Edge Add-ons, Firefox AMO; Safari → the app's
 * download page), or the project's download page when unknown.
 */
export function installUrl(): string {
  throw new Error("unimplemented: SPEC.md §7");
}
