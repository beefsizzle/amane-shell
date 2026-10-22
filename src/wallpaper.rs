use std::time::Duration;

use amane::{Animation, Image, Palette, Service};

// the wallpaper path lives in ~/.local/state/amane-shell/wallpaper, so it survives a restart
pub struct Wallpaper {
    pub path: String,             // the file that was chosen
    pub shown: String,            // the file that's drawn: a screen-sized copy, empty until it exists
    pub incoming: Option<String>, // a new copy fading in over `shown`
    pub fade: Animation,          // how far it has, 0 to 1
}

// how long a new wallpaper takes to fade in
const FADE: Duration = Duration::from_millis(450);

// a copy that never decodes is faded in anyway after this long (and shows as nothing)
const DECODE_WITHIN: Duration = Duration::from_secs(3);

fn state_file() -> std::path::PathBuf {
    crate::cache::state_dir().join("wallpaper")
}

fn initial() -> String {
    // it used to be kept with the settings
    crate::cache::move_from(&crate::cache::config_dir().join("wallpaper"), &state_file());

    if let Ok(path) = std::env::var("AMANE_WALLPAPER")
        && !path.is_empty()
    {
        return path;
    }

    std::fs::read_to_string(state_file()).map(|text| text.trim().to_string()).unwrap_or_default()
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

        Self { path, shown, incoming: None, fade: Animation::new(0.0).duration(FADE) }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

/*
 * the screen-sized copy of the wallpaper exists now: fade it in if it's still the one chosen.
 * Amane draws nothing for an image until it has been decoded, so swapping the picture straight away would show an
 * empty frame first. the old wallpaper stays until the new one is decoded, then the new one fades in over it.
 * decoding takes a moment, so this waits on a thread of its own
 */
pub fn display_ready(path: &str, display: &str) {
    /*
     * only for the chosen wallpaper: asking Amane whether an image is loaded also decodes it, and a decoded image
     * that is never drawn is never let go. the background conversions call this for every wallpaper they make
     */
    if Wallpaper::read().path != path {
        return;
    }

    let (path, display) = (path.to_string(), display.to_string());

    std::thread::spawn(move || {
        let started = std::time::Instant::now();

        // the files this thread asked about, which it lets go of again if it gives up
        let mut asked: Vec<String> = Vec::new();

        /*
         * the fade is drawn by the shell, and needs both pictures decoded: the new one, and the one under it, which
         * the compositor has been drawing from its own copy. that's whatever is on screen when the fade starts, so a
         * fade still running finishes first. asking also starts the decode. a file that won't decode is faded in
         * anyway after a while, and shows as nothing
         */
        loop {
            let (chosen, old, busy) = {
                let wallpaper = Wallpaper::read();

                (wallpaper.path == path, wallpaper.shown.clone(), wallpaper.incoming.is_some())
            };

            // a wallpaper picked meanwhile takes over
            if !chosen {
                return let_go(&asked);
            }

            let waited_out = started.elapsed() >= DECODE_WITHIN;

            if !busy && old == display {
                return let_go(&asked);
            }

            if !busy && (waited_out || (ready(&display, &mut asked) && ready(&old, &mut asked))) {
                /*
                 * checked again and written under one lock: picking another wallpaper meanwhile must not end up
                 * showing this one, the picture under the fade must still be the one on screen, and a wait that gave
                 * up (see let_go) must not have let go of either picture in between
                 */
                let mut wallpaper = Wallpaper::write();

                if wallpaper.path != path {
                    drop(wallpaper);

                    return let_go(&asked);
                }

                let decoded = waited_out || (ready(&display, &mut asked) && ready(&old, &mut asked));

                if decoded && wallpaper.shown == old && wallpaper.incoming.is_none() {
                    wallpaper.incoming = Some(display.clone());
                    wallpaper.fade = Animation::new(0.0).duration(FADE);
                    wallpaper.fade.to(1.0);

                    break;
                }
            }

            std::thread::sleep(Duration::from_millis(16));
        }

        // the colours change as the picture starts to
        retheme(&path, &display);

        std::thread::sleep(FADE);

        // once it covers the old one it becomes the wallpaper
        let mut wallpaper = Wallpaper::write();

        if wallpaper.incoming.as_deref() == Some(display.as_str()) {
            wallpaper.shown = display;
            wallpaper.incoming = None;
        }
    });
}

// whether the file is decoded, starting the decode if not, and noting it was asked about (see let_go)
fn ready(file: &str, asked: &mut Vec<String>) -> bool {
    if file.is_empty() {
        return true;
    }

    if !asked.iter().any(|known| known == file) {
        asked.push(file.to_string());
    }

    Image::loaded(file)
}

/*
 * a decoded picture no window draws is never let go by itself, so a wait that gave up lets go of what it asked to be
 * decoded. while a fade is starting or running, the two pictures in it are about to be drawn and stay. under the lock,
 * so no other wait starts a fade in between (it checks again under the lock too)
 */
fn let_go(files: &[String]) {
    let wallpaper = Wallpaper::read();

    for file in files {
        let fading = wallpaper.incoming.is_some() && (*file == wallpaper.shown || wallpaper.incoming.as_ref() == Some(file));

        if !fading {
            Image::forget(file);
        }
    }
}

// the palette is read from the small thumbnail when there is one, which is quicker than any wallpaper
fn retheme(path: &str, display: &str) {
    let source = crate::wallpapers::thumbnail_if_ready(path).unwrap_or_else(|| display.to_string());

    Palette::write().open(&source, 16);

    // and other apps follow
    crate::recolor::export();
}

/// chooses a wallpaper, remembers it for the next start, and draws it and re-themes the shell from it
/// as soon as its screen-sized copy exists (at once if it was made before, otherwise in a moment)
pub fn apply(path: &str) -> bool {
    if !std::path::Path::new(path).exists() {
        return false;
    }

    Wallpaper::write().path = path.to_string();

    // a failed write only means it won't survive a restart
    let file = state_file();

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
