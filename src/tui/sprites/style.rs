use ratatui::style::Color;

/// Shared sprite shading tokens.
///
/// `highlight`, `base`, `midtone`, `shadow`, and `outline` define the shading
/// ladder; `comb`, `beak`, `feet`, and `eye` are the accent tokens that sprite
/// modules use to keep parts visually distinct while sharing a common palette
/// contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpritePalette {
    pub highlight: Color,
    pub base: Color,
    pub midtone: Color,
    pub shadow: Color,
    pub outline: Color,
    pub comb: Color,
    pub beak: Color,
    pub feet: Color,
    pub eye: Color,
}

pub fn leader_palette() -> SpritePalette {
    SpritePalette {
        highlight: Color::Rgb(252, 248, 241),
        base: Color::Rgb(242, 234, 220),
        midtone: Color::Rgb(225, 213, 194),
        shadow: Color::Rgb(196, 182, 160),
        outline: Color::Rgb(150, 136, 120),
        comb: Color::Rgb(228, 74, 88),
        beak: Color::Rgb(250, 184, 72),
        feet: Color::Rgb(234, 158, 64),
        eye: Color::Rgb(26, 24, 30),
    }
}

pub fn chick_palette() -> SpritePalette {
    SpritePalette {
        highlight: Color::Rgb(255, 242, 132),
        base: Color::Rgb(248, 223, 92),
        midtone: Color::Rgb(234, 199, 70),
        shadow: Color::Rgb(194, 157, 52),
        outline: Color::Rgb(132, 110, 52),
        comb: Color::Rgb(248, 196, 88),
        beak: Color::Rgb(250, 180, 70),
        feet: Color::Rgb(236, 156, 60),
        eye: Color::Rgb(34, 30, 22),
    }
}

pub fn egg_palette() -> SpritePalette {
    SpritePalette {
        highlight: Color::Rgb(248, 244, 235),
        base: Color::Rgb(234, 228, 214),
        midtone: Color::Rgb(216, 207, 188),
        shadow: Color::Rgb(184, 171, 149),
        outline: Color::Rgb(138, 128, 110),
        comb: Color::Rgb(184, 171, 149),
        beak: Color::Rgb(184, 171, 149),
        feet: Color::Rgb(184, 171, 149),
        eye: Color::Rgb(34, 30, 22),
    }
}

#[cfg(test)]
mod tests {
    use super::{chick_palette, egg_palette, leader_palette};

    #[test]
    fn leader_palette_has_distinct_accents() {
        let palette = leader_palette();
        assert_ne!(palette.base, palette.highlight);
        assert_ne!(palette.comb, palette.beak);
        assert_ne!(palette.shadow, palette.base);
    }

    #[test]
    fn chick_and_egg_palettes_are_not_identical() {
        let chick = chick_palette();
        let egg = egg_palette();
        assert_ne!(chick.base, egg.base);
        assert_ne!(chick.shadow, egg.shadow);
    }
}
