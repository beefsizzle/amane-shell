#!/bin/sh
# installs daru's login screen for SDDM, once per machine:   sudo sddm/install.sh
# and puts the previous one back:                           sudo sddm/install.sh --remove
#
# the theme goes to /usr/share/sddm/themes/daru. the greeter runs as its own user and can't read your home folder, so
# the shell copies the wallpaper and its colours to /var/lib/daru-sddm, a folder this makes for you (add `sddm` to
# ~/.config/amane-shell/recolor). the theme reads them through a link, theme.conf.user, so nothing needs sudo again
set -eu

theme=/usr/share/sddm/themes/daru
data=/var/lib/daru-sddm
conf=/etc/sddm.conf
backup=/etc/sddm.conf.before-daru

if [ "$(id -u)" != 0 ]; then
    echo "run this with sudo"
    exit 1
fi

user="${SUDO_USER:-}"

if [ -z "$user" ] || [ "$user" = root ]; then
    echo "run this with sudo from your own account: the folder the shell writes to belongs to you"
    exit 1
fi

here="$(cd "$(dirname "$0")" && pwd)"

# the Current= of [Theme] in a config file, empty when there's none
theme_in() {
    sed -n '/^\[Theme\]/,/^\[/s/^Current=//p' "$1" 2>/dev/null | head -n 1
}

# /etc/sddm.conf is read after /etc/sddm.conf.d and wins, and an empty Current= there would hide one set elsewhere,
# so the theme is set in it
set_theme() {
    if [ -f "$conf" ] && grep -q '^\[Theme\]' "$conf"; then
        if sed -n '/^\[Theme\]/,/^\[/p' "$conf" | grep -q '^Current='; then
            sed -i "/^\[Theme\]/,/^\[/s/^Current=.*/Current=$1/" "$conf"
        else
            sed -i "/^\[Theme\]/a Current=$1" "$conf"
        fi
    else
        printf '\n[Theme]\nCurrent=%s\n' "$1" >>"$conf"
    fi
}

if [ "${1:-}" = "--remove" ]; then
    # only a plain theme name goes back into the config
    previous="$(theme_in "$backup" | tr -cd 'A-Za-z0-9._-')"

    set_theme "$previous"
    rm -rf "$theme" "$data"

    echo "removed. SDDM's theme is back to '${previous:-its built-in one}'"
    exit 0
fi

for file in Main.qml metadata.desktop theme.conf; do
    if [ ! -f "$here/$file" ]; then
        echo "$here/$file is missing"
        exit 1
    fi
done

# the config as it was before the first install, for --remove
if [ ! -f "$backup" ]; then
    if [ -f "$conf" ]; then cp -p "$conf" "$backup"; else : >"$backup"; fi
fi

install -d -o "$user" -m 755 "$data"

install -d -m 755 "$theme"
install -m 644 "$here/Main.qml" "$here/metadata.desktop" "$here/theme.conf" "$theme/"
ln -sfn "$data/theme.conf.user" "$theme/theme.conf.user"

set_theme daru

# a shell that's running writes the wallpaper and colours now, rather than at the next start or wallpaper change.
# it listens on a socket in the user's runtime folder
uid="$(id -u "$user")"
amane="$(getent passwd "$user" | cut -d: -f6)/.cargo/bin/amane"

if [ -x "$amane" ] && grep -qx sddm "$(getent passwd "$user" | cut -d: -f6)/.config/amane-shell/recolor" 2>/dev/null; then
    runuser -u "$user" -- env XDG_RUNTIME_DIR="/run/user/$uid" "$amane" ipc call recolor >/dev/null 2>&1 || true
fi

echo "installed. add 'sddm' to ~/.config/amane-shell/recolor and pick a wallpaper (or restart the shell), and the"
echo "login screen follows it from then on. to look at it without logging out:"
echo "    sddm-greeter-qt6 --test-mode --theme $theme"
