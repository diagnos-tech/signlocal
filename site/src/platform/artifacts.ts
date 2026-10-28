// The file names of a release. They follow packaging (docs/architecture/packaging-and-release.md)
// and the `expected=` list of .github/workflows/release.yml, which a test compares with this one.

import type { Arch, Distro, Os } from "./types.ts";

export interface ArtifactNames {
  readonly windows: Readonly<Record<Arch, string>>;
  readonly macos: string;
  readonly deb: Readonly<Record<Arch, string>>;
  readonly rpm: Readonly<Record<Arch, string>>;
  readonly tarball: Readonly<Record<Arch, string>>;
  readonly extensionChromium: string;
  readonly extensionFirefox: string;
  readonly installSh: string;
  readonly installPs1: string;
  readonly checksums: string;
}

export function artifactNames(slug: string, version: string): ArtifactNames {
  return {
    windows: {
      x64: `${slug}-${version}-windows-x64.zip`,
      arm64: `${slug}-${version}-windows-arm64.zip`,
    },
    macos: `${slug}-${version}-macos-universal.zip`,
    deb: { x64: `${slug}_${version}_amd64.deb`, arm64: `${slug}_${version}_arm64.deb` },
    rpm: { x64: `${slug}-${version}-1.x86_64.rpm`, arm64: `${slug}-${version}-1.aarch64.rpm` },
    tarball: {
      x64: `${slug}-${version}-linux-x64.tar.gz`,
      arm64: `${slug}-${version}-linux-arm64.tar.gz`,
    },
    extensionChromium: `${slug}-extension-${version}-chromium.zip`,
    extensionFirefox: `${slug}-extension-${version}-firefox.zip`,
    installSh: "install.sh",
    installPs1: "install.ps1",
    checksums: "SHA256SUMS",
  };
}

/** Every file a release must carry, each one listed in SHA256SUMS (except SHA256SUMS itself). */
export function requiredFiles(slug: string, version: string): readonly string[] {
  const names = artifactNames(slug, version);
  return [
    ...Object.values(names.windows),
    names.macos,
    ...Object.values(names.deb),
    ...Object.values(names.rpm),
    ...Object.values(names.tarball),
    names.extensionChromium,
    names.extensionFirefox,
    names.installSh,
    names.installPs1,
  ];
}

/** The file the manual download of the app offers for a system. */
export function appFile(names: ArtifactNames, os: Os, arch: Arch, distro: Distro): string | null {
  if (os === "windows") return names.windows[arch];
  if (os === "macos") return names.macos;
  if (distro === "deb") return names.deb[arch];
  if (distro === "rpm") return names.rpm[arch];
  return null;
}
