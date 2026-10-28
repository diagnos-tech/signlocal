// The shapes of the JSON the data generators write to src/generated/ (scripts/data/).

export interface ReleaseAsset {
  readonly name: string;
  readonly url: string;
  readonly size: number;
  /** Lower-case hex SHA-256, from the release's SHA256SUMS. */
  readonly sha256: string;
}

export interface ReleaseInfo {
  /** Without the leading "v". */
  readonly version: string;
  readonly tag: string;
  readonly prerelease: boolean;
  /** ISO 8601. */
  readonly publishedAt: string;
  /** The release page on GitHub. */
  readonly url: string;
  readonly assets: Readonly<Record<string, ReleaseAsset>>;
}

export interface ReleaseData {
  /** "fixture" puts a "sample data" ribbon on every page and makes CI refuse to deploy. */
  readonly source: "github" | "fixture";
  readonly generatedAt: string;
  /** null while nothing is published: the install page says the first release is coming. */
  readonly release: ReleaseInfo | null;
}

export interface ProjectData {
  readonly name: string;
  readonly slug: string;
  readonly tagline: string;
  readonly repository: string;
  /** "owner/name" of the repository on GitHub. */
  readonly repositoryPath: string;
  /** With the trailing slash. */
  readonly homepage: string;
  readonly urlScheme: string;
  readonly minAppVersion: string;
  readonly extension: {
    readonly chromeWebStoreId: string;
    readonly edgeAddonsId: string;
    readonly firefoxAmoSlug: string;
  };
  readonly sdk: { readonly version: string; readonly budgetKb: number };
}

export interface SupportData {
  /** Chrome, Edge, Opera and Brave. */
  readonly chromium: string;
  readonly firefox: string;
  readonly macos: string;
  /** Linux distributions the end-to-end suite runs on, as display names. */
  readonly linuxTested: readonly string[];
}

export interface SnippetsData {
  readonly [name: string]: string;
}
