#!/bin/sh
# Runs after the package is installed or upgraded (deb and rpm share it).
#
# Registers the native messaging host for every user (system manifests).
# That must not fail the installation: a missing browser is not a broken
# package, and `signlocal doctor` reports what is left to do.
#
# pcscd.socket is deliberately left alone: it belongs to the pcscd package,
# whose own policy (and the administrator) decides whether it runs. The
# diagnostics window shows the command to enable it when cards need it.
set -u

if command -v signlocal >/dev/null 2>&1; then
    signlocal install --system || echo "signlocal: system registration failed; run 'sudo signlocal install --system'" >&2
fi
exit 0
