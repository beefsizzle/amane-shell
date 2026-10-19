#!/bin/sh
# Runs a command against the nested session: dev/ctl.sh <command...>
# e.g.  dev/ctl.sh notify-send hi   |   dev/ctl.sh niri msg action screenshot-screen
. "$(dirname "$0")/.envload.sh"
exec "$@"
