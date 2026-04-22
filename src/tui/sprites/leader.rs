use crate::tui::sprites::style::{leader_palette, SpritePalette};
use ratatui::style::Color;

pub type Pixel = Option<Color>;

fn leader_row(pattern: &str, palette: SpritePalette, eye: Pixel) -> Vec<Pixel> {
    debug_assert_eq!(pattern.chars().count(), 20);

    pattern
        .chars()
        .map(|cell| match cell {
            '.' => None,
            'H' => Some(palette.highlight),
            'B' => Some(palette.base),
            'M' => Some(palette.midtone),
            'S' => Some(palette.shadow),
            'O' => Some(palette.outline),
            'C' => Some(palette.comb),
            'K' => Some(palette.beak),
            'E' => eye,
            'F' => Some(palette.feet),
            _ => panic!("unknown leader sprite cell: {cell}"),
        })
        .collect()
}

pub fn leader_idle(frame: usize) -> Vec<Vec<Pixel>> {
    let palette = leader_palette();
    let eye = if frame % 18 < 2 {
        Some(palette.highlight)
    } else {
        Some(palette.eye)
    };

    [
        ".............CC.....",
        "............CCHH....",
        "....SSSS....HHHHKK..",
        "..SSSMMM...HHHEHKKK.",
        "..SSMMHHHHHHHHHHHK..",
        ".SSMMHHHHHHHHHHHH...",
        "SSMMHHHHHMMMMHHH....",
        "SSMMHHHMMMSSMMHH....",
        ".SSMMHHHMMBBBBMM....",
        "..SSMMHHHMMMBBMM....",
        "...SSMMHHHHMMMM.....",
        "....SSMMHHHHH.......",
        "......MMHHHH........",
        ".......FF..FF.......",
        "......FF....FF......",
        "....................",
    ]
    .into_iter()
    .map(|row| leader_row(row, palette, eye))
    .collect()
}

pub fn leader_peck(frame: usize) -> Vec<Vec<Pixel>> {
    if frame % 4 < 2 {
        let palette = leader_palette();
        let mut sprite = leader_idle(frame);
        sprite[2][17] = Some(palette.beak);
        sprite[2][18] = Some(palette.beak);
        sprite[3][18] = Some(palette.beak);
        sprite[3][19] = Some(palette.shadow);
        sprite
    } else {
        leader_idle(frame)
    }
}

#[allow(dead_code)]
pub fn leader_sleep(_frame: usize) -> Vec<Vec<Pixel>> {
    let palette = leader_palette();
    let mut sprite = leader_idle(0);
    sprite[3][12] = Some(palette.midtone);
    sprite[3][13] = Some(palette.midtone);
    sprite[2][13] = Some(palette.highlight);
    sprite
}

#[allow(dead_code)]
pub fn leader_done() -> Vec<Vec<Pixel>> {
    let mut sprite = leader_idle(0);
    sprite[0][15] = Some(Color::Rgb(255, 220, 80));
    sprite[1][15] = Some(Color::Rgb(255, 220, 80));
    sprite[1][16] = Some(Color::Rgb(255, 220, 80));
    sprite
}

#[cfg(test)]
mod tests {
    use super::{leader_done, leader_idle, leader_peck, leader_sleep};

    fn count_region(
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
    fn leader_states_return_pixels() {
        assert!(!leader_idle(0).is_empty());
        assert!(!leader_peck(0).is_empty());
        assert!(!leader_sleep(0).is_empty());
        assert!(!leader_done().is_empty());
    }

    #[test]
    fn leader_idle_has_distinct_tail_and_head_regions() {
        let sprite = leader_idle(0);
        let width = sprite.first().map(|row| row.len()).unwrap_or(0);
        let tail_mass = count_region(&sprite, 0..5.min(width), 1..7);
        let head_mass = count_region(&sprite, width.saturating_sub(6)..width, 1..8);

        assert!(tail_mass >= 8, "tail/back region too sparse: {tail_mass}");
        assert!(head_mass >= 12, "head/beak region too sparse: {head_mass}");
    }

    #[test]
    fn leader_idle_uses_larger_three_quarter_canvas() {
        let sprite = leader_idle(0);

        assert_eq!(sprite.len(), 16);
        assert!(sprite.iter().all(|row| row.len() == 20));
    }
}
