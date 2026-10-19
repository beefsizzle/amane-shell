use std::time::Duration;

use amane::{Palette, Service};

// the wallpaper path lives in ~/.config/amane-shell/wallpaper, so it survives a restart
pub struct Wallpaper {
    pub path: String,  // the file that was chosen
    pub shown: String, // the file that's drawn: a screen-sized copy, empty until it exists
}

fn config_file() -> std::path::PathBuf {
    crate::cache::config_dir().join("wallpaper")
}

fn initial() -> String {
    if let Ok(path) = std::env::var("AMANE_WALLPAPER")
        && !path.is_empty()
    {
        return path;
    }

    std::fs::read_to_string(config_file()).map(|text| text.trim().to_string()).unwrap_or_default()
}

// the copy when it exists, the original when Amane can read it as it is, nothing otherwise
fn shown_for(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    if let Some(display) = crate::wallpapers::display_if_ready(path) {
        return display;
    }

    // only real png and jpeg data can be drawn without conversion, whatever the extension says
    let mut start = [0u8; 8];

    let readable = std::fs::File::open(path)
        .and_then(|mut file| std::io::Read::read(&mut file, &mut start))
        .is_ok_and(|read| read >= 3 && (start.starts_with(&[0x89, b'P', b'N', b'G']) || start.starts_with(&[0xFF, 0xD8, 0xFF])));

    if readable { path.to_string() } else { String::new() }
}

impl Service for Wallpaper {
    fn new() -> Self {
        let path = initial();

        let shown = shown_for(&path);

        Self { path, shown }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

/// the screen-sized copy of the wallpaper exists now, draw it if that's the one that was chosen
pub fn display_ready(path: &str, display: &str) {
    // checked and written under one lock: picking another wallpaper in between must not end up showing this one
    let chosen = {
        let mut wallpaper = Wallpaper::write();

        let chosen = wallpaper.path == path && wallpaper.shown != display;

        if chosen {
            wallpaper.shown = display.to_string();
        }

        chosen
    };

    // and the colours only if it's still the one chosen
    if chosen && Wallpaper::read().path == path {
        retheme(path, display);
    }
}

// the palette is read from the small thumbnail when there is one, which is quicker than any wallpaper
fn retheme(path: &str, display: &str) {
    let source = crate::wallpapers::thumbnail_if_ready(path).unwrap_or_else(|| display.to_string());

    Palette::write().open(&source, 16);
}

/// chooses a wallpaper, remembers it for the next start, and draws it and re-themes the shell from it
/// as soon as its screen-sized copy exists (at once if it was made before, otherwise in a moment)
pub fn apply(path: &str) -> bool {
    if !std::path::Path::new(path).exists() {
        return false;
    }

    Wallpaper::write().path = path.to_string();

    // a failed write only means it won't survive a restart
    let file = config_file();

    if let Some(folder) = std::path::Path::new(&file).parent() {
        let _ = std::fs::create_dir_all(folder);
    }

    let _ = std::fs::write(file, path);

    if let Some(display) = crate::wallpapers::display_if_ready(path) {
        display_ready(path, &display);
    } else {
        let path = path.to_string();

        // converting takes a second or two, which is too long for the thread that draws
        std::thread::spawn(move || {
            if let Some(display) = crate::wallpapers::make_display(&path) {
                display_ready(&path, &display);
            }
        });
    }

    true
}

/// `amane ipc call wallpaper <path>`
pub fn ipc(arguments: &[String]) -> String {
    let Some(path) = arguments.first() else {
        return String::from("usage: wallpaper <path>");
    };

    if apply(path) { format!("wallpaper {path}") } else { format!("no such file: {path}") }
}
