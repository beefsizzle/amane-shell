#!/bin/sh
# screenshot of the nested session: dev/shot.sh /path/to/out.png
. "$(dirname "$0")/.envload.sh"
niri msg action screenshot-screen --show-pointer false --path "${1:-/tmp/amane-shot.png}"
sleep 1
