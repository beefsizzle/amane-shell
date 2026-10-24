/*
 * what differs between the compositors the shell runs on: how to log out, how to take a screenshot, and where the
 * focused window's title comes from (see focus.rs). each sets its own variable for the programs it starts
 */
#[derive(Clone, Copy, PartialEq)]
pub enum Compositor {
    Niri,
    Umbriel,
    Other,
}

pub fn running() -> Compositor {
    if std::env::var_os("NIRI_SOCKET").is_some() {
        Compositor::Niri
    } else if std::env::var_os("UMBRIEL_SOCKET").is_some() {
        Compositor::Umbriel
    } else {
        Compositor::Other
    }
}

/// ends the session without asking
pub fn log_out() -> &'static str {
    match running() {
        Compositor::Niri => "niri msg action quit --skip-confirmation",
        Compositor::Umbriel => "umbriel msg session-quit:skip-confirmation",
        Compositor::Other => "loginctl terminate-session \"$XDG_SESSION_ID\"",
    }
}

/// what `amane ipc call screenshot region|screen|annotate` takes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shot {
    Region,   // a region you drag out
    Screen,   // the monitor in focus
    Annotate, // a region, opened in satty to draw on first
}

impl Shot {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "region" => Some(Self::Region),
            "screen" => Some(Self::Screen),
            "annotate" => Some(Self::Annotate),
            _ => None,
        }
    }
}

// where every screenshot goes, as niri saves them
const SAVE: &str = "folder=\"$HOME/Pictures/Screenshots\"; mkdir -p \"$folder\"; \
                    file=\"$folder/Screenshot from $(date '+%Y-%m-%d %H-%M-%S').png\"";

// copied, and a notification with the picture in it, as niri does
const DONE: &str = "wl-copy --type image/png < \"$file\" && \
                    notify-send --app-name=Screenshot --icon=\"$file\" \"Screenshot taken\" \"Saved in Pictures/Screenshots and copied\"";

/*
 * the command for a screenshot. niri takes its own, which save, copy and notify by themselves; elsewhere it's grim and
 * slurp doing the same, and satty for drawing on one. a cancelled drag does nothing
 */
pub fn screenshot(shot: Shot) -> String {
    match (running(), shot) {
        (Compositor::Niri, Shot::Region) => String::from("niri msg action screenshot"),
        (Compositor::Niri, Shot::Screen) => String::from("niri msg action screenshot-screen"),
        (_, Shot::Region) => format!("region=\"$(slurp)\" || exit; {SAVE}; grim -g \"$region\" \"$file\" && {DONE}"),
        (Compositor::Umbriel, Shot::Screen) => {
            format!("output=\"$(umbriel outputs --json | jq -r '.[] | select(.focused) | .name')\"; {SAVE}; grim -o \"$output\" \"$file\" && {DONE}")
        }
        (_, Shot::Screen) => format!("{SAVE}; grim \"$file\" && {DONE}"),
        // satty saves, copies and notifies by itself (see ~/.config/satty/config.toml)
        (_, Shot::Annotate) => String::from("region=\"$(slurp)\" || exit; grim -g \"$region\" - | satty --filename -"),
    }
}

#[cfg(test)]
mod tests {
    use super::Shot;

    #[test]
    fn shots_by_name() {
        assert_eq!(Shot::parse("region"), Some(Shot::Region));
        assert_eq!(Shot::parse("screen"), Some(Shot::Screen));
        assert_eq!(Shot::parse("annotate"), Some(Shot::Annotate));
        assert_eq!(Shot::parse("window"), None);
    }
}
