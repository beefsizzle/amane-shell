# daru

A desktop shell for [niri](https://github.com/YaLTeR/niri), built with [Amane](https://github.com/MystiaFin/amane). Bar, tray,
dashboard, launcher, notifications, calendar and weather, wallpaper picker, and sound, Bluetooth, network, clipboard and
power dropdowns. Colours follow the wallpaper.

![screenshot](media/screenshot.jpg)





https://github.com/user-attachments/assets/a843d4c3-b076-4d88-abff-8120f4c5d30a






## Requirements

- niri
- the [Amane](https://github.com/MystiaFin/amane) CLI. This `upstream` branch builds with upstream Amane, and does without
  what the [`main`](https://github.com/beefsizzle/daru) branch gets from [my fork](https://github.com/beefsizzle/amane):
  fingerprint unlock, a wallpaper drawn by the compositor, and lower GPU memory
- Adwaita Sans, JetBrainsMono Nerd Font
- ImageMagick, wl-clipboard, cliphist, `pactl` (PipeWire or PulseAudio), NetworkManager, BlueZ, curl, jq

## Install

```sh
git clone https://github.com/MystiaFin/amane
cd amane && ./install.sh && cd ..

git clone -b upstream https://github.com/beefsizzle/daru
amane target daru
amane compile
amane run
```

Stop any other bar, notification daemon or tray host first.

## niri

```kdl
spawn-at-startup "amane" "run"
spawn-at-startup "wl-paste" "--type" "text" "--watch" "cliphist" "-max-items" "200" "store"
spawn-at-startup "wl-paste" "--type" "image" "--watch" "cliphist" "-max-items" "200" "store"

binds {
    Mod+Space   { spawn "amane" "ipc" "call" "toggle" "launcher"; }
    Mod+Shift+D { spawn "amane" "ipc" "call" "toggle" "dashboard"; }
    Mod+Shift+C { spawn "amane" "ipc" "call" "toggle" "calendar"; }
    Mod+Shift+N { spawn "amane" "ipc" "call" "toggle" "notifications"; }
    Mod+Shift+W { spawn "amane" "ipc" "call" "toggle" "wallpapers"; }
    Mod+Ctrl+V  { spawn "amane" "ipc" "call" "toggle" "clipboard"; }
    Alt+X       { spawn "amane" "ipc" "call" "toggle" "power"; }
    Super+Alt+L { spawn "amane" "ipc" "call" "lock"; }
}
```

Blur behind the bar and panels: add [`dev/niri-layer-rules.kdl`](dev/niri-layer-rules.kdl).

## Other apps' colours

The shell can write the wallpaper colours for other apps, live where the app allows it. List the apps in
`~/.config/amane-shell/recolor`, one per line, and point each app at its file once:

| app | file it writes | add to the app's config |
|---|---|---|
| `gtk` (GTK4 / libadwaita) | `~/.config/gtk-4.0/daru.css` (read when an app starts), and GNOME's `accent-color` (live, nearest of its 9) | `@import url("daru.css");` in `~/.config/gtk-4.0/gtk.css` |
| `niri` (focus ring) | `~/.config/niri/daru-colors.kdl` | `include "daru-colors.kdl"` at the end of `config.kdl` |
| `ghostty` | `~/.config/ghostty/daru-colors` | `config-file = ?daru-colors` |
| `zed` | `~/.config/zed/themes/daru.json` | pick the "Daru" theme |
| `kde` (Dolphin, Qt apps) | `~/.local/share/color-schemes/Daru.colors`, and the colours in `~/.config/kdeglobals` (live) | install `plasma-integration` and set `QT_QPA_PLATFORMTHEME=kde` |

## Config

In `~/.config/amane-shell/`:

- `location`: `lat lon`, for the weather
- `wallpaper-dir`: wallpaper folder, default `~/Pictures/Wallpapers`

## IPC

```
amane ipc call toggle dashboard|calendar|notifications|wallpapers|launcher|power|network|audio|bluetooth|clipboard|all|close
amane ipc call wallpaper <path>
amane ipc call lock
amane ipc call sleep
```

## License

[MIT](LICENSE)
