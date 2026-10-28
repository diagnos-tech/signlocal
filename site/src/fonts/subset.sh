#!/usr/bin/env sh
# Rebuilds the two web font files from their upstream releases.
#
# Why subsets: the first screen of every page waits for the fonts, and the home page is
# measured on a slow connection. The site uses weights 400 to 700 only and seven Latin-script
# languages, so each family keeps those weights (the optical-size axis of Inter stays), the
# Latin characters, punctuation and arrows the pages can show, and the few OpenType features
# the styles rely on (kerning, ligatures, tabular figures, case-sensitive forms). Hinting is
# dropped: browsers on every desktop and phone rasterize without it.
#
# Needs: curl, tar, and fonttools with brotli (`pip install fonttools brotli`).
# OFL-1.1 allows modified versions; neither family declares a Reserved Font Name.
# The unicode-range in ../styles/fonts.css lists the same characters.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/inter" "$work/mono"
curl -fsSL https://registry.npmjs.org/@fontsource-variable/inter/-/inter-5.3.0.tgz |
  tar xz -C "$work/inter"
curl -fsSL https://registry.npmjs.org/@fontsource-variable/jetbrains-mono/-/jetbrains-mono-5.3.0.tgz |
  tar xz -C "$work/mono"

ranges='U+0020-007E,U+00A0-00FF,U+0152-0153,U+0178,U+2013-2014,U+2018-201E,U+2022,U+2026,U+2039-203A,U+20AC,U+2122,U+2190-2193,U+2212'
features='kern,liga,calt,ccmp,locl,mark,mkmk,tnum,case'

build() {
  source=$1
  target=$2
  python3 -m fontTools.varLib.instancer "$source" wght=400:700 --output "$work/narrow.woff2"
  python3 -m fontTools.subset "$work/narrow.woff2" --unicodes="$ranges" \
    --layout-features="$features" --no-hinting --flavor=woff2 --output-file="$target"
}

build "$work/inter/package/files/inter-latin-opsz-normal.woff2" "$here/inter-subset.woff2"
build "$work/mono/package/files/jetbrains-mono-latin-wght-normal.woff2" \
  "$here/jetbrains-mono-subset.woff2"
