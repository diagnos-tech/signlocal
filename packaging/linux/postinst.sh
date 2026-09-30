#!/bin/sh
# Runs after the package is installed or upgraded (deb and rpm share it).
#
# Registers the native messaging host for every user (system manifests) and
# enables the PC/SC socket so smart cards work without a manual step. Neither
# may fail the installation: a missing browser or pcscd is not a broken
# package, and `websign doctor` reports what is left to do.
set -u

if command -v websign >/dev/null 2>&1; then
    websign install --system || echo "websign: system registration failed; run 'sudo websign install --system'" >&2
fi

if command -v systemctl >/dev/null 2>&1 && [ -d /run/systemd/system ]; then
    # Only when pcscd is present: it is a recommendation, not a dependency.
    if systemctl list-unit-files pcscd.socket >/dev/null 2>&1; then
        systemctl enable --now pcscd.socket >/dev/null 2>&1 || true
    fi
fi
exit 0
