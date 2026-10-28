// Looking up a file of the release for a component: a missing one is a bug in the data, so it
// throws instead of rendering a broken download.

import type { ReleaseAsset, ReleaseInfo } from "./data-types.ts";

export function assetOf(release: ReleaseInfo, name: string): ReleaseAsset {
  const asset = release.assets[name];
  if (!asset) throw new Error(`The release ${release.tag} has no file ${name}`);
  return asset;
}
