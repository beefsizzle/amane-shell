// the bar's battery icon (see bar.rs), from amane's Battery service, which reads /sys/class/power_supply every 5 s

/// at or below this a battery that's running down shows in red
pub const LOW: u8 = 15;

/// the icon for a charge from 0 to 100: the level in tenths while it runs down, a bolt in it while it charges, and a
/// plug when it's plugged in and full
pub fn glyph(percent: u8, charging: bool, full: bool) -> &'static str {
    if full {
        return "\u{f06a5}";
    }

    if charging {
        // the font has no charging icon for 50 or 70
        return match percent {
            95.. => "\u{f0085}",
            90..95 => "\u{f008b}",
            80..90 => "\u{f008a}",
            60..80 => "\u{f0089}",
            40..60 => "\u{f0088}",
            30..40 => "\u{f0087}",
            20..30 => "\u{f0086}",
            _ => "\u{f089c}",
        };
    }

    const TENTHS: [&str; 9] = [
        "\u{f007a}", "\u{f007b}", "\u{f007c}", "\u{f007d}", "\u{f007e}", "\u{f007f}", "\u{f0080}", "\u{f0081}", "\u{f0082}",
    ];

    match percent {
        95.. => "\u{f0079}",
        10..95 => TENTHS[usize::from(percent / 10) - 1],
        _ => "\u{f0083}",
    }
}

#[cfg(test)]
mod tests {
    use super::glyph;

    #[test]
    fn running_down_shows_the_level_in_tenths() {
        assert_eq!(glyph(100, false, false), "\u{f0079}");
        assert_eq!(glyph(57, false, false), "\u{f007e}");
        assert_eq!(glyph(10, false, false), "\u{f007a}");
    }

    #[test]
    fn nearly_empty_shows_a_warning() {
        assert_eq!(glyph(9, false, false), "\u{f0083}");
        assert_eq!(glyph(0, false, false), "\u{f0083}");
    }

    #[test]
    fn charging_shows_a_bolt_and_full_on_the_charger_a_plug() {
        assert_eq!(glyph(99, true, false), "\u{f0085}");
        assert_eq!(glyph(55, true, false), "\u{f0088}");
        assert_eq!(glyph(3, true, false), "\u{f089c}");
        assert_eq!(glyph(100, false, true), "\u{f06a5}");
    }
}
