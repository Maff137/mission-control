#!/usr/bin/env bash
# Run only the acceptance controller/app in an empty network namespace.
# Preserve the same unprivileged user and Unix-domain X11/DBus access.
set -euo pipefail
exec sudo unshare --net -- runuser -u "$(id -un)" -- env \
  "HOME=$HOME" "PATH=$PATH" "DISPLAY=$DISPLAY" "XAUTHORITY=$XAUTHORITY" \
  "DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS" MC_NETWORK_ISOLATED=1 \
  python scripts/runtime-acceptance.py run "$1"
