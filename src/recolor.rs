use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use amane::{Argument, Bus, Color, Value};

use crate::theme::{Theme, hex, mix, over, signal, theme};

/*
 * the wallpaper's colours for other apps, written whenever the shell's own colours change. each app reads its file
 * from its own config, so it has to be pointed at it once (see the README); the shell only ever writes its own files
 * and never edits an app's config. which apps get a file is up to ~/.config/amane-shell/recolor, one name per line:
 *
 *     gtk       ~/.config/gtk-4.0/daru.css, for GTK4 and libadwaita apps when they next start
 *     niri      ~/.config/niri/daru-colors.kdl, the focus ring. niri reloads it by itself
 *     ghostty   ~/.config/ghostty/daru-colors, background, text, cursor and selection. Ghostty is asked to reload
 *     zed       ~/.config/zed/themes/daru.json, a theme called "Daru". Zed reloads it by itself
 */
#[derive(Clone, Copy, PartialEq)]
enum App {
    Gtk,
    Niri,
    Ghostty,
    Zed,
}

// one export at a time: two wallpaper changes in quick succession must not interleave their writes
static WRITING: Mutex<()> = Mutex::new(());

fn wanted() -> Vec<App> {
    let text = std::fs::read_to_string(crate::cache::config_dir().join("recolor")).unwrap_or_default();

    text.lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter_map(|name| match name {
            "gtk" => Some(App::Gtk),
            "niri" => Some(App::Niri),
            "ghostty" => Some(App::Ghostty),
            "zed" => Some(App::Zed),
            _ => None,
        })
        .collect()
}

/// writes the current colours for every app that asked for them. the files are small, but this runs on a thread of
/// its own all the same, after the shell has already recoloured itself
pub fn export() {
    std::thread::spawn(|| {
        let _writing = crate::sync::lock(&WRITING);

        let apps = wanted();

        if apps.is_empty() {
            return;
        }

        let colors = Colors::from(&theme());

        for app in apps {
            match app {
                App::Gtk => {
                    write(&config("gtk-4.0/daru.css"), &gtk(&colors));
                }
                App::Niri => {
                    write(&config("niri/daru-colors.kdl"), &niri(&colors));
                }
                App::Ghostty => {
                    if write(&config("ghostty/daru-colors"), &ghostty(&colors)) {
                        reload_ghostty();
                    }
                }
                App::Zed => {
                    write(&config("zed/themes/daru.json"), &zed(&colors));
                }
            }
        }
    });
}

fn config(file: &str) -> PathBuf {
    crate::cache::config_dir().parent().map_or_else(PathBuf::new, Path::to_path_buf).join(file)
}

// written under another name and renamed, so an app never reads half a file. false when it was already up to date,
// so nothing needs to reload
fn write(path: &Path, text: &str) -> bool {
    if std::fs::read_to_string(path).is_ok_and(|current| current == text) {
        return false;
    }

    if let Some(folder) = path.parent() {
        let _ = std::fs::create_dir_all(folder);
    }

    let partial = path.with_extension("daru-part");

    std::fs::write(&partial, text).is_ok() && std::fs::rename(&partial, path).is_ok()
}

// the theme's colours made solid, as other apps want them, plus a few hues for syntax and the terminal
struct Colors {
    background: Color, // the panels' background
    surface: Color,    // chips and cards
    raised: Color,     // the same, hovered or selected
    border: Color,
    text: Color,
    dim: Color,
    faint: Color,
    accent: Color,
    on_accent: Color,
    red: Color,
    yellow: Color,
    green: Color,
    blue: Color,
    purple: Color,
    orange: Color,
}

impl Colors {
    fn from(t: &Theme) -> Self {
        let background = Color::rgb(t.panel.red(), t.panel.green(), t.panel.blue());

        let hue = |seed: Color| signal(seed, t.accent);

        Self {
            background,
            surface: over(t.surface, background),
            raised: over(t.surface_hi, background),
            border: over(t.border, background),
            text: t.fg,
            dim: over(t.dim, background),
            faint: over(t.faint, background),
            accent: t.accent,
            on_accent: t.on_accent,
            red: t.red,
            yellow: t.yellow,
            green: hue(Color::rgb(0x9e, 0xce, 0x6a)),
            blue: hue(Color::rgb(0x7a, 0xa2, 0xf7)),
            purple: hue(Color::rgb(0xbb, 0x9a, 0xf7)),
            orange: hue(Color::rgb(0xff, 0x9e, 0x64)),
        }
    }
}

const HEADER: &str = "written by the daru shell from the wallpaper, and rewritten on every change";

// libadwaita 1.6 and later read the css variables, older GTK4 apps the named colours
fn gtk(c: &Colors) -> String {
    let roles = [
        ("accent-bg", c.accent),
        ("accent-fg", c.on_accent),
        ("accent", c.accent),
        ("window-bg", c.background),
        ("window-fg", c.text),
        ("view-bg", c.background),
        ("view-fg", c.text),
        ("headerbar-bg", c.surface),
        ("headerbar-fg", c.text),
        ("headerbar-backdrop", c.background),
        ("sidebar-bg", c.surface),
        ("sidebar-fg", c.text),
        ("secondary-sidebar-bg", c.surface),
        ("secondary-sidebar-fg", c.text),
        ("card-bg", c.surface),
        ("card-fg", c.text),
        ("dialog-bg", c.surface),
        ("dialog-fg", c.text),
        ("popover-bg", c.raised),
        ("popover-fg", c.text),
        ("thumbnail-bg", c.surface),
        ("thumbnail-fg", c.text),
    ];

    let mut text = format!("/* {HEADER} */\n\n:root {{\n");

    for (role, color) in roles {
        text.push_str(&format!("    --{role}-color: {};\n", hex(color)));
    }

    text.push_str("}\n\n");

    for (role, color) in roles {
        text.push_str(&format!("@define-color {}_color {};\n", role.replace('-', "_"), hex(color)));
    }

    text
}

fn niri(c: &Colors) -> String {
    format!(
        "// {HEADER}\n\nlayout {{\n    focus-ring {{\n        active-color \"{}\"\n        inactive-color \"{}\"\n    }}\n}}\n",
        hex(c.accent),
        hex(c.raised),
    )
}

// only the base colours: the 16 terminal colours keep their usual hues, which programs rely on, apart from the black
// and white ones, which follow the background and the text
fn ghostty(c: &Colors) -> String {
    let lines = [
        ("background", c.background),
        ("foreground", c.text),
        ("cursor-color", c.accent),
        ("cursor-text", c.on_accent),
        ("selection-background", mix(c.background, c.accent, 0.35)),
        ("selection-foreground", c.text),
    ];

    let mut text = format!("# {HEADER}\n\n");

    for (key, color) in lines {
        text.push_str(&format!("{key} = {}\n", hex(color)));
    }

    for (index, color) in [(0, c.surface), (7, c.dim), (8, c.faint), (15, c.text)] {
        text.push_str(&format!("palette = {index}={}\n", hex(color)));
    }

    text
}

// Ghostty reloads its config when asked over D-Bus, but only while it's running: asking would otherwise start it
fn reload_ghostty() {
    const NAME: &str = "com.mitchellh.ghostty";

    let running = Bus::session()
        .call("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus", "NameHasOwner", &[Argument::Text(NAME.to_string())])
        .list()
        .first()
        .is_some_and(Value::bool);

    if !running {
        return;
    }

    // its actions take an empty list and dictionary, which gdbus writes out more simply than Amane's arguments
    let _ = Command::new("gdbus")
        .args(["call", "--session", "--dest", NAME, "--object-path", "/com/mitchellh/ghostty"])
        .args(["--method", "org.gtk.Actions.Activate", "reload-config", "[]", "{}"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn zed(c: &Colors) -> String {
    let h = |color: Color| hex(color);

    // a colour with transparency, "#rrggbbaa"
    let a = |color: Color, alpha: f32| format!("{}{:02x}", hex(color), (alpha * 255.0).round() as u8);

    let syntax = |color: Color| format!("{{ \"color\": \"{}\" }}", h(color));

    format!(
        r#"{{
  "$schema": "https://zed.dev/schema/themes/v0.2.0.json",
  "name": "Daru",
  "author": "{HEADER}",
  "themes": [
    {{
      "name": "Daru",
      "appearance": "dark",
      "style": {{
        "background": "{background}",
        "surface.background": "{surface}",
        "elevated_surface.background": "{raised}",
        "panel.background": "{surface}",
        "status_bar.background": "{background}",
        "title_bar.background": "{background}",
        "title_bar.inactive_background": "{background}",
        "toolbar.background": "{background}",
        "tab_bar.background": "{surface}",
        "tab.active_background": "{background}",
        "tab.inactive_background": "{surface}",
        "editor.background": "{background}",
        "editor.foreground": "{text}",
        "editor.gutter.background": "{background}",
        "editor.active_line.background": "{active_line}",
        "editor.line_number": "{faint}",
        "editor.active_line_number": "{text}",
        "editor.indent_guide": "{border}",
        "editor.indent_guide_active": "{faint}",
        "border": "{border}",
        "border.variant": "{border}",
        "border.focused": "{accent}",
        "border.selected": "{accent}",
        "text": "{text}",
        "text.muted": "{dim}",
        "text.placeholder": "{faint}",
        "text.accent": "{accent}",
        "icon": "{text}",
        "icon.muted": "{dim}",
        "icon.accent": "{accent}",
        "element.background": "{surface}",
        "element.hover": "{raised}",
        "element.selected": "{selected}",
        "element.active": "{selected}",
        "ghost_element.hover": "{raised}",
        "ghost_element.selected": "{selected}",
        "ghost_element.active": "{selected}",
        "scrollbar.thumb.background": "{scrollbar}",
        "scrollbar.track.background": "{background}",
        "search.match_background": "{search}",
        "error": "{red}",
        "warning": "{yellow}",
        "success": "{green}",
        "info": "{blue}",
        "created": "{green}",
        "modified": "{yellow}",
        "deleted": "{red}",
        "terminal.background": "{background}",
        "terminal.foreground": "{text}",
        "accents": ["{accent}"],
        "players": [
          {{ "cursor": "{accent}", "background": "{accent}", "selection": "{selection}" }}
        ],
        "syntax": {{
          "keyword": {purple_syntax},
          "function": {blue_syntax},
          "string": {green_syntax},
          "number": {orange_syntax},
          "constant": {orange_syntax},
          "boolean": {orange_syntax},
          "type": {yellow_syntax},
          "attribute": {yellow_syntax},
          "tag": {red_syntax},
          "property": {text_syntax},
          "variable": {text_syntax},
          "punctuation": {dim_syntax},
          "operator": {dim_syntax},
          "title": {accent_syntax},
          "link_uri": {accent_syntax},
          "comment": {{ "color": "{faint}", "font_style": "italic" }}
        }}
      }}
    }}
  ]
}}
"#,
        background = h(c.background),
        surface = h(c.surface),
        raised = h(c.raised),
        active_line = a(c.raised, 0.5),
        faint = h(c.faint),
        border = h(c.border),
        accent = h(c.accent),
        text = h(c.text),
        dim = h(c.dim),
        selected = a(c.accent, 0.25),
        scrollbar = a(c.text, 0.2),
        search = a(c.accent, 0.3),
        selection = a(c.accent, 0.3),
        red = h(c.red),
        yellow = h(c.yellow),
        green = h(c.green),
        blue = h(c.blue),
        purple_syntax = syntax(c.purple),
        blue_syntax = syntax(c.blue),
        green_syntax = syntax(c.green),
        orange_syntax = syntax(c.orange),
        yellow_syntax = syntax(c.yellow),
        red_syntax = syntax(c.red),
        text_syntax = syntax(c.text),
        dim_syntax = syntax(c.dim),
        accent_syntax = syntax(c.accent),
    )
}
