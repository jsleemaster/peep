use crate::tui::sprites::style::leader_palette;
use ratatui::style::Color;

pub type Pixel = Option<Color>;

fn n() -> Pixel {
    None
}

pub fn leader_idle(frame: usize) -> Vec<Vec<Pixel>> {
    let palette = leader_palette();
    let h = Some(palette.highlight);
    let b = Some(palette.base);
    let m = Some(palette.midtone);
    let s = Some(palette.shadow);
    let comb = Some(palette.comb);
    let beak = Some(palette.beak);
    let eye = if frame % 18 < 2 { h } else { Some(palette.eye) };
    let feet = Some(palette.feet);
    let n = n();

    vec![
        vec![n, n, n, comb, comb, n, n, n, n, n, n, n, n, n, n, n],
        vec![n, n, h, h, h, h, h, n, n, n, n, n, n, n, n, n],
        vec![n, h, h, eye, h, h, h, beak, beak, n, n, n, n, n, n, n],
        vec![n, h, h, h, h, h, h, h, m, n, n, n, n, n, n, n],
        vec![n, n, h, h, h, h, h, h, h, h, n, n, n, n, n, n],
        vec![n, h, h, h, h, h, h, h, h, h, h, n, n, n, n, n],
        vec![h, h, h, h, m, m, h, h, h, h, h, h, n, n, n, n],
        vec![h, h, h, m, s, m, m, h, h, h, h, h, h, n, n, n],
        vec![h, h, h, m, m, b, h, h, h, h, h, h, h, n, n, n],
        vec![n, h, h, h, h, h, m, m, h, h, h, h, n, n, n, n],
        vec![n, n, h, h, h, h, h, h, h, h, h, n, n, n, n, n],
        vec![n, n, n, h, h, h, h, h, h, h, n, n, n, n, n, n],
        vec![n, n, n, n, feet, feet, n, n, feet, feet, n, n, n, n, n, n],
        vec![
            n, n, n, feet, feet, n, n, feet, feet, feet, n, n, n, n, n, n,
        ],
    ]
}

pub fn leader_peck(frame: usize) -> Vec<Vec<Pixel>> {
    if frame % 4 < 2 {
        let palette = leader_palette();
        let mut sprite = leader_idle(frame);
        sprite[1][2] = None;
        sprite[1][3] = Some(palette.comb);
        sprite[2][7] = Some(palette.beak);
        sprite[2][8] = Some(palette.beak);
        sprite[3][8] = Some(palette.shadow);
        sprite
    } else {
        leader_idle(frame)
    }
}

#[allow(dead_code)]
pub fn leader_sleep(_frame: usize) -> Vec<Vec<Pixel>> {
    let palette = leader_palette();
    let mut sprite = leader_idle(0);
    let highlight = Some(palette.highlight);
    sprite[2][3] = highlight;
    sprite[2][4] = highlight;
    sprite[3][3] = Some(palette.midtone);
    sprite[3][4] = Some(palette.midtone);
    sprite
}

#[allow(dead_code)]
pub fn leader_done() -> Vec<Vec<Pixel>> {
    let mut sprite = leader_idle(0);
    sprite[0][10] = Some(Color::Rgb(255, 220, 80));
    sprite[1][10] = Some(Color::Rgb(255, 220, 80));
    sprite[1][11] = Some(Color::Rgb(255, 220, 80));
    sprite
}

#[cfg(test)]
mod tests {
    use super::{leader_done, leader_idle, leader_peck, leader_sleep};

    #[test]
    fn leader_states_return_pixels() {
        assert!(!leader_idle(0).is_empty());
        assert!(!leader_peck(0).is_empty());
        assert!(!leader_sleep(0).is_empty());
        assert!(!leader_done().is_empty());
    }
}
