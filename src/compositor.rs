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

/*
 * a screenshot of a region you drag out. niri has its own; elsewhere it's grim and slurp, saved where niri saves
 * them and copied to the clipboard like niri does. a cancelled drag saves nothing
 */
pub fn screenshot_region() -> &'static str {
    match running() {
        Compositor::Niri => "niri msg action screenshot",
        _ => "region=\"$(slurp)\" || exit; \
              folder=\"$HOME/Pictures/Screenshots\"; mkdir -p \"$folder\"; \
              file=\"$folder/Screenshot from $(date '+%Y-%m-%d %H-%M-%S').png\"; \
              grim -g \"$region\" \"$file\" && wl-copy --type image/png < \"$file\"",
    }
}
