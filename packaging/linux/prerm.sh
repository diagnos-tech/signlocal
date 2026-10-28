#!/bin/sh
# Runs before the package is removed or upgraded (deb and rpm share it).
#
# Removes the system registrations only on a real removal: an upgrade keeps
# them (the new postinst rewrites them). deb passes "remove"/"upgrade"; rpm
# passes the number of package instances left (0 on removal).
set -u

case "${1:-}" in
    remove | purge | 0) ;;
    *) exit 0 ;;
esac

if command -v signlocal >/dev/null 2>&1; then
    signlocal uninstall --system || echo "signlocal: removing the system registrations failed; browser manifests may remain (docs/install.md, Uninstall)" >&2
fi
exit 0
