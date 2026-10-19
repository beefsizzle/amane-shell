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

    // the darkest wallpaper color, pushed further toward black so text always reads
    let base = mix(palette.background(), Color::BLACK, 0.35);
    let fg = mix(palette.foreground(), Color::WHITE, 0.55);
    let accent = palette.accent();

    Theme {
        glass: alpha(base, 0.62),
        panel: alpha(base, 0.78),
        surface: alpha(mix(base, fg, 0.10), 0.80),
        surface_hi: alpha(mix(base, fg, 0.20), 0.90),
        border: alpha(fg, 0.10),
        fg,
        dim: alpha(fg, 0.62),
        faint: alpha(fg, 0.38),
        accent,
        accent_soft: alpha(accent, 0.22),
        on_accent: palette.on_accent(),
        red: Color::from("#f7768e"),
        yellow: Color::from("#e0af68"),
    }
}
