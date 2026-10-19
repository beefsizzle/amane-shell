use amane::{Color, Palette, Service};

pub const TEXT_FONT: &str = "Adwaita Sans";
pub const ICON_FONT: &str = "JetBrainsMono Nerd Font";

// corner radii. the bar chips are square on purpose. Raise CHIP_RADIUS to round them again (13 is a full pill)
pub const CHIP_RADIUS: f32 = 0.0;
pub const RADIUS_SMALL: f32 = 0.0;

// every color the shell uses, derived from the wallpaper through `Palette`
#[derive(Clone, Copy)]
pub struct Theme {
    pub glass: Color,       // translucent base behind the blur
    pub panel: Color,       // popup background, a little more opaque than the bar
    pub surface: Color,     // chips and cards on top of a panel
    pub surface_hi: Color,  // the same, hovered
    pub border: Color,      // hairlines
    pub fg: Color,
    pub dim: Color,
    pub faint: Color,
    pub accent: Color,
    pub accent_soft: Color, // accent tint for active chips
    pub on_accent: Color,
    pub red: Color,
    pub yellow: Color,
}

pub fn alpha(color: Color, amount: f32) -> Color {
    Color::rgba(color.red(), color.green(), color.blue(), (amount.clamp(0.0, 1.0) * 255.0) as u8)
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let lerp = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;

    Color::rgba(
        lerp(a.red(), b.red()),
        lerp(a.green(), b.green()),
        lerp(a.blue(), b.blue()),
        lerp(a.alpha(), b.alpha()),
    )
}

pub fn theme() -> Theme {
    let palette = Palette::read();

    derive(palette.background(), palette.foreground(), palette.accent())
}

/*
 * every role has a fixed lightness, and the wallpaper only lends its hues (and some of its saturation). mixing the
 * wallpaper's own colours toward black and white made the bar's depth and contrast depend on the picture: a wallpaper
 * whose darkest colour is a mid grey gave a light bar with weak text. this way every wallpaper reads the same and
 * only the colour changes
 */
const BACKGROUND: f32 = 0.075; // the glass and panels
const SURFACE: f32 = 0.15; // chips and cards
const SURFACE_HI: f32 = 0.22; // the same, hovered
const TEXT: f32 = 0.92;
const ACCENT: f32 = 0.68; // bright enough to read as text on the background
const ON_ACCENT: f32 = 0.12; // text on an accent fill
const SIGNAL: f32 = 0.70; // red and yellow

// a hue is only kept when there's colour to keep: a grey has none, and its hue would turn it red
const GREY: f32 = 0.08;

// warnings start from these and lean a little toward the accent, so they belong to the wallpaper's colours
const RED_SEED: Color = Color::rgb(0xf7, 0x76, 0x8e);
const YELLOW_SEED: Color = Color::rgb(0xe0, 0xaf, 0x68);

pub fn derive(background: Color, foreground: Color, accent: Color) -> Theme {
    let base = |lightness: f32| relit(background, lightness, 0.0, 0.35);

    let fg = relit(foreground, TEXT, 0.0, 0.12);
    let accent_color = relit(accent, ACCENT, 0.45, 0.82);

    let signal = |seed: Color| relit(mix(seed, accent_color, 0.20), SIGNAL, 0.48, 0.82);

    Theme {
        glass: alpha(base(BACKGROUND), 0.62),
        panel: alpha(base(BACKGROUND), 0.78),
        surface: alpha(base(SURFACE), 0.80),
        surface_hi: alpha(base(SURFACE_HI), 0.90),
        border: alpha(fg, 0.10),
        fg,
        dim: alpha(fg, 0.62),
        faint: alpha(fg, 0.38),
        accent: accent_color,
        accent_soft: alpha(accent_color, 0.22),
        on_accent: relit(accent, ON_ACCENT, 0.0, 0.50),
        red: signal(RED_SEED),
        yellow: signal(YELLOW_SEED),
    }
}

// the colour's hue at the given lightness, its saturation kept within `least..=most` (a grey stays grey)
fn relit(color: Color, lightness: f32, least: f32, most: f32) -> Color {
    let (hue, saturation, _) = to_hsl(color);

    let saturation = if saturation < GREY { saturation } else { saturation.clamp(least, most) };

    from_hsl(hue, saturation, lightness)
}

// hue in degrees, saturation and lightness 0..1
fn to_hsl(color: Color) -> (f32, f32, f32) {
    let [r, g, b] = [color.red(), color.green(), color.blue()].map(|channel| f32::from(channel) / 255.0);

    let (high, low) = (r.max(g).max(b), r.min(g).min(b));
    let lightness = (high + low) / 2.0;
    let spread = high - low;

    if spread == 0.0 {
        return (0.0, 0.0, lightness);
    }

    let saturation = spread / (1.0 - (2.0 * lightness - 1.0).abs());

    let hue = if high == r {
        60.0 * ((g - b) / spread).rem_euclid(6.0)
    } else if high == g {
        60.0 * ((b - r) / spread + 2.0)
    } else {
        60.0 * ((r - g) / spread + 4.0)
    };

    (hue, saturation.min(1.0), lightness)
}

fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> Color {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let second = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
    let offset = lightness - chroma / 2.0;

    let (r, g, b) = match hue.rem_euclid(360.0) {
        h if h < 60.0 => (chroma, second, 0.0),
        h if h < 120.0 => (second, chroma, 0.0),
        h if h < 180.0 => (0.0, chroma, second),
        h if h < 240.0 => (0.0, second, chroma),
        h if h < 300.0 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };

    let channel = |value: f32| ((value + offset).clamp(0.0, 1.0) * 255.0).round() as u8;

    Color::rgb(channel(r), channel(g), channel(b))
}

#[cfg(test)]
mod tests {
    use super::{BACKGROUND, Color, derive, from_hsl, to_hsl};

    // WCAG contrast between two opaque colours
    fn contrast(a: Color, b: Color) -> f32 {
        let luminance = |color: Color| {
            let linear = |channel: u8| {
                let value = f32::from(channel) / 255.0;

                if value <= 0.04045 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
            };

            0.2126 * linear(color.red()) + 0.7152 * linear(color.green()) + 0.0722 * linear(color.blue())
        };

        let (light, dark) = (luminance(a).max(luminance(b)), luminance(a).min(luminance(b)));

        (light + 0.05) / (dark + 0.05)
    }

    fn opaque(color: Color) -> Color {
        Color::rgb(color.red(), color.green(), color.blue())
    }

    // wallpapers of every kind: a dark desert, a pale grey, a vivid magenta, and a black and white one
    const WALLPAPERS: [(Color, Color, Color); 4] = [
        (Color::rgb(0x2a, 0x1a, 0x10), Color::rgb(0xf0, 0xd0, 0xa0), Color::rgb(0xe0, 0x90, 0x40)),
        (Color::rgb(0x88, 0x88, 0x8c), Color::rgb(0xee, 0xee, 0xee), Color::rgb(0x70, 0x90, 0xb0)),
        (Color::rgb(0x40, 0x00, 0x30), Color::rgb(0xff, 0xc0, 0xf0), Color::rgb(0xff, 0x00, 0xc0)),
        (Color::rgb(0x00, 0x00, 0x00), Color::rgb(0xff, 0xff, 0xff), Color::rgb(0x80, 0x80, 0x80)),
    ];

    #[test]
    fn hsl_round_trips() {
        for color in [Color::rgb(0xf7, 0x76, 0x8e), Color::rgb(0x12, 0x34, 0x56), Color::rgb(0x80, 0x80, 0x80), Color::rgb(0, 0xff, 0)] {
            let (hue, saturation, lightness) = to_hsl(color);
            let back = from_hsl(hue, saturation, lightness);

            for (a, b) in [(color.red(), back.red()), (color.green(), back.green()), (color.blue(), back.blue())] {
                assert!(a.abs_diff(b) <= 1, "{color:?} came back as {back:?}");
            }
        }
    }

    #[test]
    fn every_wallpaper_gets_the_same_depth() {
        for (background, foreground, accent) in WALLPAPERS {
            let (_, _, lightness) = to_hsl(opaque(derive(background, foreground, accent).panel));

            assert!((lightness - BACKGROUND).abs() < 0.01, "panel lightness {lightness}");
        }
    }

    #[test]
    fn text_and_accent_read_on_every_wallpaper() {
        for (background, foreground, accent) in WALLPAPERS {
            let theme = derive(background, foreground, accent);
            let panel = opaque(theme.panel);

            assert!(contrast(theme.fg, panel) >= 12.0, "text contrast {}", contrast(theme.fg, panel));
            assert!(contrast(theme.accent, panel) >= 6.0, "accent contrast {}", contrast(theme.accent, panel));
            assert!(contrast(theme.on_accent, theme.accent) >= 4.5, "on-accent contrast {}", contrast(theme.on_accent, theme.accent));
            assert!(contrast(theme.red, panel) >= 4.5, "red contrast {}", contrast(theme.red, panel));
        }
    }

    #[test]
    fn a_grey_wallpaper_stays_grey() {
        let theme = derive(Color::rgb(0x50, 0x50, 0x50), Color::rgb(0xdd, 0xdd, 0xdd), Color::rgb(0x99, 0x99, 0x99));
        let panel = opaque(theme.panel);

        assert!(panel.red() == panel.green() && panel.green() == panel.blue(), "{panel:?}");
    }
}
