#!/usr/bin/env sh
# Rebuilds the embedded font files from their upstream releases.
#
# Why subsets: the app embeds its fonts, so every glyph costs binary size.
# Inter keeps Latin (all seven UI languages), Greek and Cyrillic (the window
# shows look-alike IDN hosts "as displayed"), punctuation and arrows; the mono
# font only renders codes, paths and IDs, so ASCII is enough. Hinting is
# dropped because egui rasterizes without it.
#
# Needs: curl, tar, and fonttools with brotli (`pip install fonttools brotli`).
# Output replaces the .ttf files next to this script. OFL-1.1 allows modified
# versions; neither family declares a Reserved Font Name.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

curl -fsSL https://registry.npmjs.org/inter-font/-/inter-font-3.19.0.tgz | tar xz -C "$work"
mkdir -p "$work/jbm"
curl -fsSL https://registry.npmjs.org/@fontsource/jetbrains-mono/-/jetbrains-mono-5.3.0.tgz |
  tar xz -C "$work/jbm"

names='0,1,2,3,4,5,6,13,14'
inter_ranges='U+0020-007E,U+00A0-017F,U+0192,U+0218-021B,U+02C6-02DD,U+0370-03FF,U+0400-045F,U+0490-0491,U+1E9E,U+2000-206F,U+20AC,U+2122,U+2190-2199,U+2212,U+2215,U+2713,U+FFFD'
for weight in Regular Medium SemiBold; do
  pyftsubset "$work/package/ttf/Inter-$weight.ttf" --unicodes="$inter_ranges" \
    --no-hinting --layout-features=kern --name-IDs="$names" \
    --output-file="$here/Inter-$weight.ttf"
done

mono_ranges='U+0020-007E,U+00A0,U+00B7,U+2022,U+2026'
for pair in 400:Regular 500:Medium; do
  pyftsubset "$work/jbm/package/files/jetbrains-mono-latin-${pair%%:*}-normal.woff2" \
    --unicodes="$mono_ranges" --no-hinting --layout-features='' --name-IDs="$names" \
    --flavor= --output-file="$here/JetBrainsMono-${pair##*:}.ttf"
done
