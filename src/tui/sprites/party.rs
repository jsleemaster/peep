use crate::tui::sprites::style::{chick_palette, egg_palette};
use ratatui::style::Color;

pub type Pixel = Option<Color>;

fn n() -> Pixel {
    None
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
    let h = Some(p.highlight);
    let b = Some(p.base);
    let m = Some(p.midtone);
    let s = Some(p.shadow);
    let n = n();

    vec![
        vec![n, n, n, n, n, n],
        vec![n, n, h, h, n, n],
        vec![n, h, h, b, b, n],
        vec![n, h, b, b, m, n],
        vec![b, b, b, b, m, s],
        vec![n, b, b, m, s, n],
        vec![n, n, s, s, n, n],
    ]
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
    egg[0][2] = Some(p.base);
    egg[1][2] = Some(p.base);
    egg[1][3] = Some(p.eye);
    egg[1][4] = Some(p.beak);
    egg
}

pub fn party_walking(frame: usize) -> Vec<Vec<Pixel>> {
    let p = chick_palette();
    let h = Some(p.highlight);
    let b = Some(p.base);
    let m = Some(p.midtone);
    let e = if frame % 12 < 2 { h } else { Some(p.eye) };
    let beak = Some(p.beak);
    let feet = Some(p.feet);
    let n = n();

    if frame % 6 < 3 {
        vec![
            vec![n, n, b, b, b, b, n, n],
            vec![n, b, b, b, b, b, b, n],
            vec![b, b, e, b, b, b, beak, beak],
            vec![b, b, b, b, b, b, b, n],
            vec![n, b, m, b, b, m, b, n],
            vec![feet, feet, n, feet, feet, n, n, n],
        ]
    } else {
        vec![
            vec![n, n, b, b, b, b, n, n],
            vec![n, b, b, b, b, b, b, n],
            vec![b, b, e, b, b, b, beak, beak],
            vec![b, b, b, b, b, b, b, n],
            vec![b, m, b, b, m, b, n, n],
            vec![n, feet, feet, n, feet, feet, n, n],
        ]
    }
}

pub fn party_sleeping(_frame: usize) -> Vec<Vec<Pixel>> {
    let mut sprite = party_walking(0);
    sprite[2][2] = Some(chick_palette().highlight);
    sprite[2][4] = Some(chick_palette().highlight);
    sprite
}

pub fn party_done() -> Vec<Vec<Pixel>> {
    let mut sprite = party_walking(0);
    sprite[0][6] = Some(Color::Rgb(255, 220, 80));
    sprite[1][6] = Some(Color::Rgb(255, 220, 80));
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
}
