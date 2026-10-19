#!/bin/sh
# Runs a sandboxed niri in a window, with its own D-Bus so notifications/MPRIS don't
# collide with the real session, and its own XDG_RUNTIME_DIR for the shell so it can run
# next to the real one. Usage: dev/nested.sh [wallpaper]
cd "$(dirname "$0")" || exit 1

mkdir -p /tmp/amane-nested-runtime && chmod 700 /tmp/amane-nested-runtime

export AMANE_WALLPAPER="${1:-$HOME/Pictures/Wallpapers/465731-beautiful-minimalist-desktop-wallpaper-4k-catppuccin-mocha.png}"

# the shell binary is copied, so `amane restart` (which kills by path) never touches the real one
cp "$HOME/.cache/amane/amane-shell" "$PWD/.amane-shell-nested"

exec dbus-run-session -- niri -c "$PWD/niri-nested.kdl"
