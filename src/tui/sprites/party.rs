use crate::tui::sprites::style::{chick_palette, egg_palette};
use ratatui::style::Color;

pub type Pixel = Option<Color>;

fn party_row(
    pattern: &str,
    highlight_eye: Pixel,
    palette: crate::tui::sprites::style::SpritePalette,
) -> Vec<Pixel> {
    pattern
        .chars()
        .map(|cell| match cell {
            '.' => None,
            'H' => Some(palette.highlight),
            'B' => Some(palette.base),
            'M' => Some(palette.midtone),
            'S' => Some(palette.shadow),
            'O' => Some(palette.outline),
            'K' => Some(palette.beak),
            'F' => Some(palette.feet),
            'E' => highlight_eye,
            _ => panic!("unknown party sprite cell: {cell}"),
        })
        .collect()
}

pub fn growth_stage(usage_count: u64, is_done: bool) -> &'static str {
    if is_done {
        "done"
    } else if usage_count >= 20 {
        "chick"
    } else if usage_count >= 10 {
        "peeking"
    } else if usage_count >= 5 {
        "hatching"
    } else {
        "egg"
    }
}

pub fn party_egg() -> Vec<Vec<Pixel>> {
    let p = egg_palette();

    [
        "........",
        "...HH...",
        "..HHBB..",
        ".HBBBB..",
        ".BBBBMS.",
        ".BBBBMS.",
        "..BBSS..",
        "...SS...",
    ]
    .into_iter()
    .map(|row| party_row(row, Some(p.eye), p))
    .collect()
}

pub fn party_hatching(frame: usize) -> Vec<Vec<Pixel>> {
    let mut egg = party_egg();
    let crack = Some(egg_palette().outline);

    if frame % 6 < 3 {
        egg[2][2] = crack;
        egg[3][3] = crack;
        egg[4][1] = crack;
    } else {
        egg[2][3] = crack;
        egg[3][2] = crack;
        egg[4][4] = crack;
    }

    egg
}

pub fn party_peeking(frame: usize) -> Vec<Vec<Pixel>> {
    let mut egg = party_hatching(frame);
    let p = chick_palette();
    egg[0][3] = Some(p.base);
    egg[0][4] = Some(p.base);
    egg[1][3] = Some(p.base);
    egg[1][4] = Some(p.eye);
    egg[1][5] = Some(p.beak);
    egg
}

pub fn party_walking(frame: usize) -> Vec<Vec<Pixel>> {
    let p = chick_palette();
    let eye = if frame % 12 < 2 {
        Some(p.highlight)
    } else {
        Some(p.eye)
    };

    if frame % 6 < 3 {
        [
            "...BBBB...",
            "..BBBBBB..",
            ".BBBEBBBKK",
            "BBBBBBBBB.",
            "..MBBBBM..",
            "..FF.FF...",
        ]
        .into_iter()
        .map(|row| party_row(row, eye, p))
        .collect()
    } else {
        [
            "...BBBB...",
            "..BBBBBB..",
            ".BBBEBBBKK",
            "BBBBBBBBB.",
            ".MBBBBBM..",
            ".FF...FF..",
        ]
        .into_iter()
        .map(|row| party_row(row, eye, p))
        .collect()
    }
}

pub fn party_sleeping(_frame: usize) -> Vec<Vec<Pixel>> {
    let mut sprite = party_walking(0);
    sprite[2][4] = Some(chick_palette().highlight);
    sprite[2][5] = Some(chick_palette().highlight);
    sprite
}

pub fn party_done() -> Vec<Vec<Pixel>> {
    let mut sprite = party_walking(0);
    sprite[0][7] = Some(Color::Rgb(255, 220, 80));
    sprite[1][7] = Some(Color::Rgb(255, 220, 80));
    sprite
}

pub use party_done as chick_done;
pub use party_egg as egg_sprite;
pub use party_hatching as egg_cracking;
pub use party_peeking as egg_hatching_chick;
pub use party_sleeping as chick_sleeping;
pub use party_walking as chick_sprite;

#[cfg(test)]
mod tests {
    use super::{
        growth_stage, party_done, party_egg, party_hatching, party_peeking, party_sleeping,
        party_walking,
    };

    fn row_width(sprite: &[Vec<super::Pixel>], row: usize) -> usize {
        let Some(row) = sprite.get(row) else {
            return 0;
        };
        let occupied: Vec<bool> = row.iter().map(Option::is_some).collect();
        let Some(left) = occupied.iter().position(|cell| *cell) else {
            return 0;
        };
        let right = occupied.iter().rposition(|cell| *cell).unwrap();
        right - left + 1
    }

    fn region_count(
        sprite: &[Vec<super::Pixel>],
        col_range: std::ops::Range<usize>,
        row_range: std::ops::Range<usize>,
    ) -> usize {
        row_range
            .filter_map(|row| sprite.get(row))
            .map(|row| {
                col_range
                    .clone()
                    .filter(|&col| row.get(col).copied().flatten().is_some())
                    .count()
            })
            .sum()
    }

    #[test]
    fn growth_stage_matches_thresholds() {
        assert_eq!(growth_stage(0, false), "egg");
        assert_eq!(growth_stage(5, false), "hatching");
        assert_eq!(growth_stage(10, false), "peeking");
        assert_eq!(growth_stage(20, false), "chick");
        assert_eq!(growth_stage(2, true), "done");
    }

    #[test]
    fn party_states_return_pixels() {
        assert!(!party_egg().is_empty());
        assert!(!party_hatching(0).is_empty());
        assert!(!party_peeking(0).is_empty());
        assert!(!party_walking(0).is_empty());
        assert!(!party_sleeping(0).is_empty());
        assert!(!party_done().is_empty());
    }

    #[test]
    fn party_egg_uses_wider_canvas_for_rounder_silhouette() {
        let sprite = party_egg();

        assert_eq!(sprite.len(), 8);
        assert!(sprite.iter().all(|row| row.len() == 8));
        assert!(row_width(&sprite, 1) <= 2);
        assert!(row_width(&sprite, 4) >= 6);
    }

    #[test]
    fn party_chick_has_clear_front_head_region() {
        let sprite = party_walking(0);
        let head_mass = region_count(&sprite, 6..10, 0..4);

        assert!(head_mass >= 10, "head/front region too small: {head_mass}");
    }
}
