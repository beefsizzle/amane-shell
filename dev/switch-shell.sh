#!/bin/sh
# switch between the two shells without touching the login setup: dev/switch-shell.sh noctalia|amane
# (amane starts at login from the niri config, noctalia.service is disabled)
export PATH="$HOME/.cargo/bin:$PATH"
case "$1" in
    noctalia)
        pkill -x amane-shell
        sleep 1
        systemctl --user start noctalia.service
        ;;
    amane)
        systemctl --user stop noctalia.service
        sleep 1
        amane run
        ;;
    *) echo "usage: $0 noctalia|amane"; exit 1 ;;
esac
