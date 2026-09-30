#!/bin/sh
# Tests for install.sh against a fake release folder (WEBSIGN_RELEASE_DIR), so
# no network and no real install happens. Runs on Linux and macOS:
#   sh scripts/install/tests/install_sh_test.sh
set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
SCRIPT="$HERE/../install.sh"
ROOT=$(mktemp -d "${TMPDIR:-/tmp}/websign-install-test.XXXXXX")
trap 'rm -rf "$ROOT"' EXIT INT TERM
FAILED=0
VERSION=9.9.9

pass() { printf 'ok   %s\n' "$1"; }
fail() { printf 'FAIL %s\n' "$1"; FAILED=1; }
check() { # description, command...
    desc=$1; shift
    if "$@" >/dev/null 2>&1; then pass "$desc"; else fail "$desc"; fi
}
same() { # description, expected, actual
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (got $3)"; fi
}
refuse() { # description, command...: the command must fail
    desc=$1; shift
    if "$@" >/dev/null 2>&1; then fail "$desc"; else pass "$desc"; fi
}

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
    else shasum -a 256 "$1" | awk '{print $1}'; fi
}

# A fake `websign` that records its arguments in $FAKE_LOG.
write_fake_binary() {
    cat >"$1" <<'FAKE'
#!/bin/sh
echo "$*" >>"${FAKE_LOG:-/dev/null}"
FAKE
    chmod 0755 "$1"
}

# Builds the release folder $1 with the Linux tarball and the macOS zip.
make_release() {
    rel=$1
    mkdir -p "$rel" "$ROOT/pkg/websign-$VERSION-linux-x64/bin" \
        "$ROOT/pkg/websign-$VERSION-linux-x64/share/applications" \
        "$ROOT/pkg/websign-$VERSION-linux-x64/share/doc/websign" \
        "$ROOT/pkg/websign-$VERSION-linux-x64/share/icons/hicolor/48x48/apps" \
        "$ROOT/app/WebeSign.app/Contents/MacOS"
    write_fake_binary "$ROOT/pkg/websign-$VERSION-linux-x64/bin/websign"
    printf '[Desktop Entry]\nExec=websign\n' >"$ROOT/pkg/websign-$VERSION-linux-x64/share/applications/websign.desktop"
    echo license >"$ROOT/pkg/websign-$VERSION-linux-x64/share/doc/websign/LICENSE"
    echo png >"$ROOT/pkg/websign-$VERSION-linux-x64/share/icons/hicolor/48x48/apps/websign.png"
    echo deb >"$rel/websign_${VERSION}_amd64.deb"
    tar -czf "$rel/websign-$VERSION-linux-x64.tar.gz" -C "$ROOT/pkg" "websign-$VERSION-linux-x64"
    write_fake_binary "$ROOT/app/WebeSign.app/Contents/MacOS/websign"
    (cd "$ROOT/app" && zip -q -r -y "$rel/websign-$VERSION-macos-universal.zip" WebeSign.app)
    : >"$rel/SHA256SUMS"
    for f in "$rel"/websign[-_]*; do
        printf '%s  %s\n' "$(sha256_of "$f")" "$(basename "$f")" >>"$rel/SHA256SUMS"
    done
}

# Runs install.sh as Linux on a distro that has no package we can use.
run_linux() {
    WEBSIGN_RELEASE_DIR=$REL WEBSIGN_UNAME_S=Linux WEBSIGN_UNAME_M=x86_64 \
        WEBSIGN_OS_RELEASE=$ROOT/os-release.arch HOME=$ROOT/home FAKE_LOG=$ROOT/log \
        sh "$SCRIPT" "$@"
}

# ---- detection ------------------------------------------------------------

# shellcheck source=/dev/null
lib() { (WEBSIGN_INSTALL_LIB=1 . "$SCRIPT"; "$@"); }

for pair in x86_64:x64 amd64:x64 aarch64:arm64 arm64:arm64; do
    m=${pair%:*}; want=${pair#*:}
    got=$(WEBSIGN_UNAME_M=$m lib detect_arch)
    same "arch $m -> $want" "$want" "$got"
done
refuse "unknown arch is refused" env WEBSIGN_UNAME_M=riscv64 WEBSIGN_INSTALL_LIB=1 sh -c ". '$SCRIPT'; detect_arch"

distro() { # name, content, expected
    printf '%s\n' "$2" >"$ROOT/os-release.$1"
    got=$(WEBSIGN_OS_RELEASE=$ROOT/os-release.$1 lib detect_distro)
    same "distro $1 -> $3" "$3" "$got"
}
distro ubuntu 'NAME="Ubuntu"
ID=ubuntu
ID_LIKE=debian' deb
distro debian 'ID=debian' deb
distro mint 'ID=linuxmint
ID_LIKE="ubuntu debian"' deb
distro fedora 'ID=fedora' rpm
distro rocky 'ID="rocky"
ID_LIKE="rhel centos fedora"' rpm
distro arch 'ID=arch' other
# shellcheck disable=SC2016  # the quotes keep the command substitution literal
distro hostile 'ID=arch
PRETTY_NAME=$(touch '"$ROOT"'/pwned)' other
check "os-release is parsed, never executed" test ! -e "$ROOT/pwned"

# ---- install --------------------------------------------------------------

REL=$ROOT/release
make_release "$REL"

check "linux tarball install succeeds" run_linux --version "$VERSION" --yes --prefix "$ROOT/prefix"
check "binary installed and executable" test -x "$ROOT/prefix/bin/websign"
check "share tree installed" test -f "$ROOT/prefix/share/doc/websign/LICENSE"
check "desktop entry points at the installed binary" grep -q "^Exec=\"$ROOT/prefix/bin/websign\"\$" "$ROOT/prefix/share/applications/websign.desktop"
check "websign install was run" grep -q '^install$' "$ROOT/log"

check "a leading v and a second run (upgrade) are fine" run_linux --version "v$VERSION" --yes --prefix "$ROOT/prefix"

: >"$ROOT/log"
run_linux --version "$VERSION" --yes --prefix "$ROOT/prefix2" --no-register >/dev/null 2>&1
check "--no-register skips websign install" test ! -s "$ROOT/log"

run_linux --version "$VERSION" --prefix "$ROOT/prefix3" --dry-run >/dev/null 2>&1
check "--dry-run installs nothing" test ! -e "$ROOT/prefix3/bin/websign"

# ---- fail closed ----------------------------------------------------------

BAD=$ROOT/bad
cp -R "$REL" "$BAD"
printf 'tampered' >>"$BAD/websign-$VERSION-linux-x64.tar.gz"
refuse "checksum mismatch fails" env REL="$BAD" WEBSIGN_RELEASE_DIR="$BAD" WEBSIGN_UNAME_S=Linux WEBSIGN_UNAME_M=x86_64 \
    WEBSIGN_OS_RELEASE="$ROOT/os-release.arch" HOME="$ROOT/home" sh "$SCRIPT" --version "$VERSION" --yes --prefix "$ROOT/prefix4"
check "nothing is installed after a mismatch" test ! -e "$ROOT/prefix4/bin/websign"

UNLISTED=$ROOT/unlisted
cp -R "$REL" "$UNLISTED"
grep -v 'linux-x64' "$REL/SHA256SUMS" >"$UNLISTED/SHA256SUMS" || true
refuse "a file missing from SHA256SUMS fails" env WEBSIGN_RELEASE_DIR="$UNLISTED" WEBSIGN_UNAME_S=Linux WEBSIGN_UNAME_M=x86_64 \
    WEBSIGN_OS_RELEASE="$ROOT/os-release.arch" HOME="$ROOT/home" sh "$SCRIPT" --version "$VERSION" --yes --prefix "$ROOT/prefix5"

refuse "--version is required" run_linux --yes --prefix "$ROOT/prefix6"
refuse "unknown options are refused" run_linux --frobnicate

# ---- uninstall ------------------------------------------------------------

: >"$ROOT/log"
run_linux --uninstall --yes --prefix "$ROOT/prefix" >/dev/null 2>&1
check "uninstall ran websign uninstall" grep -q '^uninstall$' "$ROOT/log"
check "uninstall removed the binary" test ! -e "$ROOT/prefix/bin/websign"
check "uninstall removed the docs" test ! -e "$ROOT/prefix/share/doc/websign"
check "uninstall removed the icons" test ! -e "$ROOT/prefix/share/icons/hicolor/48x48/apps/websign.png"
check "a second uninstall is harmless" run_linux --uninstall --yes --prefix "$ROOT/prefix"

# ---- system package (deb) -------------------------------------------------

# Fake sudo, apt-get and websign on PATH record what the installer asks for.
FAKEBIN=$ROOT/fakebin
mkdir -p "$FAKEBIN"
printf '#!/bin/sh\nexec "$@"\n' >"$FAKEBIN/sudo"
cat >"$FAKEBIN/apt-get" <<'FAKE'
#!/bin/sh
echo "apt-get $*" >>"$FAKE_LOG"
FAKE
write_fake_binary "$FAKEBIN/websign"
chmod 0755 "$FAKEBIN/sudo" "$FAKEBIN/apt-get"
# Only ever called through `check`, which shellcheck cannot follow: 0.9 reports
# the body as unreachable (SC2317), 0.10+ the function as unused (SC2329).
# shellcheck disable=SC2317,SC2329
run_ubuntu() {
    PATH="$FAKEBIN:$PATH" WEBSIGN_RELEASE_DIR=$REL WEBSIGN_UNAME_S=Linux WEBSIGN_UNAME_M=x86_64 \
        WEBSIGN_OS_RELEASE=$ROOT/os-release.ubuntu HOME=$ROOT/home FAKE_LOG=$ROOT/log \
        sh "$SCRIPT" "$@" </dev/null
}

: >"$ROOT/log"
check "--yes on Ubuntu installs the per-user tarball" run_ubuntu --version "$VERSION" --yes --prefix "$ROOT/prefix7"
check "--yes never implies the system package" test ! -e "$ROOT/log" -o -z "$(grep apt-get "$ROOT/log")"
check "the tarball landed" test -x "$ROOT/prefix7/bin/websign"

: >"$ROOT/log"
check "--system installs the verified deb" run_ubuntu --version "$VERSION" --yes --system
check "the package manager got the deb" grep -q "^apt-get install -y .*/websign_${VERSION}_amd64.deb\$" "$ROOT/log"

refuse "--system without a usable package manager fails" run_linux --version "$VERSION" --system --prefix "$ROOT/prefix8"
check "nothing is installed after --system fails" test ! -e "$ROOT/prefix8/bin/websign"

# ---- macOS ----------------------------------------------------------------

if command -v unzip >/dev/null 2>&1 || command -v ditto >/dev/null 2>&1; then
    run_mac() {
        WEBSIGN_RELEASE_DIR=$REL WEBSIGN_UNAME_S=Darwin WEBSIGN_UNAME_M=arm64 \
            HOME=$ROOT/machome FAKE_LOG=$ROOT/maclog sh "$SCRIPT" "$@"
    }
    check "macOS app install succeeds" run_mac --version "$VERSION" --yes --prefix "$ROOT/Applications"
    check "app bundle installed" test -x "$ROOT/Applications/WebeSign.app/Contents/MacOS/websign"
    check "websign is linked into ~/.local/bin" test -L "$ROOT/machome/.local/bin/websign"
    check "websign install was run (macOS)" grep -q '^install$' "$ROOT/maclog"
    run_mac --uninstall --yes --prefix "$ROOT/Applications" >/dev/null 2>&1
    check "macOS uninstall removes the app and the link" test ! -e "$ROOT/Applications/WebeSign.app" -a ! -L "$ROOT/machome/.local/bin/websign"
else
    echo "skip macOS install tests (no unzip)"
fi

exit "$FAILED"
