# exports the variables the nested niri session published, but only the ones that matter
dir="$(dirname "$0")"
for name in WAYLAND_DISPLAY NIRI_SOCKET DBUS_SESSION_BUS_ADDRESS XDG_CURRENT_DESKTOP XDG_RUNTIME_DIR AMANE_WALLPAPER; do
    value="$(grep "^$name=" "$dir/.env" | head -1 | cut -d= -f2-)"
    [ -n "$value" ] && export "$name=$value"
done
export PATH="$HOME/.cargo/bin:$PATH"
