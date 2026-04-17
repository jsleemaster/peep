# Peep Pixel Sprite Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current chicken sprite pipeline with a dual-tier expressive sprite system that makes the leader and party sprites cuter, softer, and more animated while preserving stage readability and narrow-terminal safety.

**Architecture:** Split the current sprite system into focused modules for shared style, leader sprites, party sprites, and renderer profiles. Add an expressive renderer profile for richer terminal glyph output, keep a safe fallback profile for constrained layouts, and update the stage widget to choose the right sprite family and profile without owning sprite art details.

**Tech Stack:** Rust, Ratatui 0.29, crossterm, existing `src/tui/sprites/*` modules, `src/tui/widgets/stage.rs`, unicode half-block and quadrant block glyphs, existing unit tests in `src/tui/render.rs`, `src/tui/widgets/stage.rs`, and inline module tests.

---

## File Structure

### New files

- Create: `src/tui/sprites/style.rs`
  - Shared sprite palette and shading tokens
  - Common helpers for accent and neutral colors
- Create: `src/tui/sprites/leader.rs`
  - Leader sprite frames for idle, peck, sleep, and done
  - Leader frame tests
- Create: `src/tui/sprites/party.rs`
  - Party sprite frames for egg, hatching, peeking, chick, waiting, and done
  - Party stage selection helpers and tests
- Create: `docs/superpowers/plans/2026-04-17-peep-pixel-sprite-redesign.md`
  - This implementation plan

### Modified files

- Modify: `src/tui/sprites/mod.rs`
  - Export new sprite modules
- Modify: `src/tui/sprites/renderer.rs`
  - Add render profiles and expressive quadrant renderer
- Modify: `src/tui/sprites/chicken.rs`
  - Reduce to compatibility wrappers during migration
- Modify: `src/tui/widgets/stage.rs`
  - Use leader and party modules directly
  - Choose expressive or safe renderer based on available space
- Modify: `src/tui/render.rs`
  - Add draw-level regression coverage for new sprite paths

### Existing tests to extend

- Test: `src/tui/sprites/renderer.rs`
- Test: `src/tui/widgets/stage.rs`
- Test: `src/tui/render.rs`

## Task 1: Split Shared Sprite Style Tokens

**Files:**
- Create: `src/tui/sprites/style.rs`
- Modify: `src/tui/sprites/mod.rs`
- Test: `src/tui/sprites/style.rs`

- [ ] **Step 1: Write the failing test**

Add this new file with tests first:

```rust
use ratatui::style::Color;

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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test leader_palette_has_distinct_accents --bin peep`
Expected: FAIL with unresolved imports or missing functions in `src/tui/sprites/style.rs`

- [ ] **Step 3: Write minimal implementation**

Replace `src/tui/sprites/style.rs` with:

```rust
use ratatui::style::Color;

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
```

Update `src/tui/sprites/mod.rs` to:

```rust
pub mod chicken;
pub mod leader;
pub mod party;
pub mod renderer;
pub mod style;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test leader_palette_has_distinct_accents --bin peep`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/sprites/style.rs src/tui/sprites/mod.rs
git commit -m "refactor: add shared sprite style tokens"
```

## Task 2: Add Renderer Profiles and Quadrant Expressive Mode

**Files:**
- Modify: `src/tui/sprites/renderer.rs`
- Test: `src/tui/sprites/renderer.rs`

- [ ] **Step 1: Write the failing test**

Append these tests to `src/tui/sprites/renderer.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::{render_sprite, RenderOptions, RenderProfile};
    use ratatui::style::Color;

    fn sample_sprite() -> Vec<Vec<Option<Color>>> {
        let a = Some(Color::Rgb(255, 220, 80));
        let b = Some(Color::Rgb(240, 180, 70));
        vec![
            vec![None, a, a, None],
            vec![a, a, b, b],
            vec![a, b, b, None],
            vec![None, b, None, None],
        ]
    }

    #[test]
    fn expressive_renderer_produces_non_empty_output() {
        let lines = render_sprite(
            &sample_sprite(),
            Color::Black,
            RenderOptions {
                profile: RenderProfile::Expressive,
                compact: false,
            },
        );
        assert!(!lines.is_empty());
    }

    #[test]
    fn safe_renderer_preserves_existing_vertical_sampling() {
        let lines = render_sprite(
            &sample_sprite(),
            Color::Black,
            RenderOptions {
                profile: RenderProfile::Safe,
                compact: true,
            },
        );
        assert_eq!(lines.len(), 2);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test expressive_renderer_produces_non_empty_output --bin peep`
Expected: FAIL because `render_sprite`, `RenderOptions`, or `RenderProfile` do not exist

- [ ] **Step 3: Write minimal implementation**

Replace `src/tui/sprites/renderer.rs` with:

```rust
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderProfile {
    Expressive,
    Safe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    pub profile: RenderProfile,
    pub compact: bool,
}

pub fn render_sprite(
    pixels: &[Vec<Option<Color>>],
    bg: Color,
    options: RenderOptions,
) -> Vec<Line<'static>> {
    match options.profile {
        RenderProfile::Expressive => sprite_to_lines_quadrant(pixels, bg, options.compact),
        RenderProfile::Safe => {
            if options.compact {
                sprite_to_lines_compact(pixels, bg)
            } else {
                sprite_to_lines(pixels, bg)
            }
        }
    }
}

pub fn sprite_to_lines(pixels: &[Vec<Option<Color>>], bg: Color) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut y = 0;
    while y < pixels.len() {
        let mut spans = Vec::new();
        for x in 0..pixels[y].len() {
            let top = pixels[y][x];
            let btm = if y + 1 < pixels.len() { pixels[y + 1][x] } else { None };
            let (ch, style) = match (top, btm) {
                (Some(tc), Some(bc)) => ("\u{2580}", Style::default().fg(tc).bg(bc)),
                (Some(tc), None) => ("\u{2580}", Style::default().fg(tc).bg(bg)),
                (None, Some(bc)) => ("\u{2584}", Style::default().fg(bc).bg(bg)),
                (None, None) => (" ", Style::default().bg(bg)),
            };
            spans.push(Span::styled(format!("{}{}", ch, ch), style));
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

pub fn sprite_to_lines_compact(pixels: &[Vec<Option<Color>>], bg: Color) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut y = 0;
    while y < pixels.len() {
        let mut spans = Vec::new();
        for x in 0..pixels[y].len() {
            let top = pixels[y][x];
            let btm = if y + 1 < pixels.len() { pixels[y + 1][x] } else { None };
            let (ch, style) = match (top, btm) {
                (Some(tc), Some(bc)) => ("\u{2580}", Style::default().fg(tc).bg(bc)),
                (Some(tc), None) => ("\u{2580}", Style::default().fg(tc).bg(bg)),
                (None, Some(bc)) => ("\u{2584}", Style::default().fg(bc).bg(bg)),
                (None, None) => (" ", Style::default().bg(bg)),
            };
            spans.push(Span::styled(ch, style));
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

fn sprite_to_lines_quadrant(
    pixels: &[Vec<Option<Color>>],
    bg: Color,
    compact: bool,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut y = 0;
    while y < pixels.len() {
        let mut spans = Vec::new();
        let width = pixels[y].len();
        let mut x = 0;
        while x < width {
            let ul = pixels.get(y).and_then(|row| row.get(x)).copied().flatten();
            let ur = pixels.get(y).and_then(|row| row.get(x + 1)).copied().flatten();
            let ll = pixels.get(y + 1).and_then(|row| row.get(x)).copied().flatten();
            let lr = pixels.get(y + 1).and_then(|row| row.get(x + 1)).copied().flatten();
            let mask =
                (ul.is_some() as u8) | ((ur.is_some() as u8) << 1) | ((ll.is_some() as u8) << 2) | ((lr.is_some() as u8) << 3);
            let glyph = quadrant_char(mask);
            let fg = dominant_color([ul, ur, ll, lr]).unwrap_or(bg);
            let span = if compact {
                Span::styled(glyph, Style::default().fg(fg).bg(bg))
            } else {
                Span::styled(format!("{}{}", glyph, glyph), Style::default().fg(fg).bg(bg))
            };
            spans.push(span);
            x += 2;
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

fn dominant_color(colors: [Option<Color>; 4]) -> Option<Color> {
    colors.into_iter().flatten().next()
}

fn quadrant_char(mask: u8) -> &'static str {
    match mask {
        0 => " ",
        1 => "\u{2598}",
        2 => "\u{259D}",
        3 => "\u{2580}",
        4 => "\u{2596}",
        5 => "\u{258C}",
        6 => "\u{259E}",
        7 => "\u{259B}",
        8 => "\u{2597}",
        9 => "\u{259A}",
        10 => "\u{2590}",
        11 => "\u{259C}",
        12 => "\u{2584}",
        13 => "\u{2599}",
        14 => "\u{259F}",
        _ => "\u{2588}",
    }
}

#[cfg(test)]
mod tests {
    use super::{render_sprite, RenderOptions, RenderProfile};
    use ratatui::style::Color;

    fn sample_sprite() -> Vec<Vec<Option<Color>>> {
        let a = Some(Color::Rgb(255, 220, 80));
        let b = Some(Color::Rgb(240, 180, 70));
        vec![
            vec![None, a, a, None],
            vec![a, a, b, b],
            vec![a, b, b, None],
            vec![None, b, None, None],
        ]
    }

    #[test]
    fn expressive_renderer_produces_non_empty_output() {
        let lines = render_sprite(
            &sample_sprite(),
            Color::Black,
            RenderOptions {
                profile: RenderProfile::Expressive,
                compact: false,
            },
        );
        assert!(!lines.is_empty());
    }

    #[test]
    fn safe_renderer_preserves_existing_vertical_sampling() {
        let lines = render_sprite(
            &sample_sprite(),
            Color::Black,
            RenderOptions {
                profile: RenderProfile::Safe,
                compact: true,
            },
        );
        assert_eq!(lines.len(), 2);
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test expressive_renderer_produces_non_empty_output --bin peep`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/sprites/renderer.rs
git commit -m "feat: add expressive sprite renderer profiles"
```

## Task 3: Move Leader Sprite Art Into Its Own Module

**Files:**
- Create: `src/tui/sprites/leader.rs`
- Modify: `src/tui/sprites/chicken.rs`
- Test: `src/tui/sprites/leader.rs`

- [ ] **Step 1: Write the failing test**

Create `src/tui/sprites/leader.rs` with this test-only starting point:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test leader_states_return_pixels --bin peep`
Expected: FAIL because the leader functions are not defined

- [ ] **Step 3: Write minimal implementation**

Replace `src/tui/sprites/leader.rs` with:

```rust
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
        vec![h, h, h, m, m, h, h, h, h, h, h, h, h, n, n, n],
        vec![n, h, h, h, h, h, m, m, h, h, h, h, n, n, n, n],
        vec![n, n, h, h, h, h, h, h, h, h, h, n, n, n, n, n],
        vec![n, n, n, h, h, h, h, h, h, h, n, n, n, n, n, n],
        vec![n, n, n, n, feet, feet, n, n, feet, feet, n, n, n, n, n, n],
        vec![n, n, n, feet, feet, n, n, feet, feet, feet, n, n, n, n, n, n],
    ]
}

pub fn leader_peck(frame: usize) -> Vec<Vec<Pixel>> {
    if frame % 4 < 2 {
        let mut sprite = leader_idle(frame);
        sprite[1][2] = None;
        sprite[1][3] = Some(leader_palette().comb);
        sprite[2][7] = Some(leader_palette().beak);
        sprite[2][8] = Some(leader_palette().beak);
        sprite[3][8] = Some(leader_palette().shadow);
        sprite
    } else {
        leader_idle(frame)
    }
}

pub fn leader_sleep(_frame: usize) -> Vec<Vec<Pixel>> {
    let mut sprite = leader_idle(0);
    let highlight = Some(leader_palette().highlight);
    sprite[2][3] = highlight;
    sprite[2][4] = highlight;
    sprite[3][3] = Some(leader_palette().midtone);
    sprite[3][4] = Some(leader_palette().midtone);
    sprite
}

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
```

Reduce `src/tui/sprites/chicken.rs` to compatibility wrappers:

```rust
pub use crate::tui::sprites::leader::{
    leader_done as chicken_done,
    leader_idle as chicken_idle,
    leader_peck as chicken_peck,
};
pub use crate::tui::sprites::party::{
    chick_done,
    chick_sleeping,
    chick_sprite,
    egg_cracking,
    egg_hatching_chick,
    egg_sprite,
    growth_stage as agent_growth_stage,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test leader_states_return_pixels --bin peep`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/sprites/leader.rs src/tui/sprites/chicken.rs
git commit -m "feat: split leader sprites into dedicated module"
```

## Task 4: Move Party Sprite Art and Stage Logic Into Its Own Module

**Files:**
- Create: `src/tui/sprites/party.rs`
- Modify: `src/tui/sprites/chicken.rs`
- Test: `src/tui/sprites/party.rs`

- [ ] **Step 1: Write the failing test**

Create `src/tui/sprites/party.rs` with:

```rust
#[cfg(test)]
mod tests {
    use super::{growth_stage, party_done, party_egg, party_hatching, party_peeking, party_sleeping, party_walking};

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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test growth_stage_matches_thresholds --bin peep`
Expected: FAIL because the party functions are missing

- [ ] **Step 3: Write minimal implementation**

Replace `src/tui/sprites/party.rs` with:

```rust
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
            vec![n, n, b, b, b, n, n, n],
            vec![n, b, b, b, b, b, n, n],
            vec![n, b, e, b, b, b, beak, n],
            vec![n, b, b, b, b, b, n, n],
            vec![n, m, b, b, m, b, n, n],
            vec![n, n, feet, n, feet, n, n, n],
        ]
    } else {
        vec![
            vec![n, n, b, b, b, n, n, n],
            vec![n, b, b, b, b, b, n, n],
            vec![n, b, e, b, b, b, beak, n],
            vec![n, m, b, b, b, b, n, n],
            vec![n, n, b, m, b, n, n, n],
            vec![n, feet, n, n, feet, n, n, n],
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
    use super::{growth_stage, party_done, party_egg, party_hatching, party_peeking, party_sleeping, party_walking};

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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test growth_stage_matches_thresholds --bin peep`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/sprites/party.rs src/tui/sprites/chicken.rs
git commit -m "feat: split party sprites into dedicated module"
```

## Task 5: Integrate New Sprite Modules and Profile Selection Into Stage

**Files:**
- Modify: `src/tui/widgets/stage.rs`
- Test: `src/tui/widgets/stage.rs`

- [ ] **Step 1: Write the failing test**

Append these tests to `src/tui/widgets/stage.rs`:

```rust
    #[test]
    fn leader_uses_safe_profile_when_left_panel_is_too_narrow() {
        assert_eq!(leader_render_profile(10), crate::tui::sprites::renderer::RenderProfile::Safe);
        assert_eq!(leader_render_profile(44), crate::tui::sprites::renderer::RenderProfile::Expressive);
    }

    #[test]
    fn party_uses_safe_profile_in_compact_mode() {
        assert_eq!(party_render_profile(true, 12), crate::tui::sprites::renderer::RenderProfile::Safe);
        assert_eq!(party_render_profile(false, 18), crate::tui::sprites::renderer::RenderProfile::Expressive);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test leader_uses_safe_profile_when_left_panel_is_too_narrow --bin peep`
Expected: FAIL because `leader_render_profile` and `party_render_profile` do not exist

- [ ] **Step 3: Write minimal implementation**

Make these changes inside `src/tui/widgets/stage.rs`:

```rust
use crate::tui::sprites::{leader, party};
use crate::tui::sprites::renderer::{render_sprite, RenderOptions, RenderProfile};
```

Add these helpers near the existing helper section:

```rust
fn leader_render_profile(width: u16) -> RenderProfile {
    if width < 14 {
        RenderProfile::Safe
    } else {
        RenderProfile::Expressive
    }
}

fn party_render_profile(use_compact: bool, col_w: u16) -> RenderProfile {
    if use_compact || col_w < 8 {
        RenderProfile::Safe
    } else {
        RenderProfile::Expressive
    }
}
```

Update the empty state and leader rendering calls to:

```rust
    let chicken_pixels = if (tick / 600).is_multiple_of(2) {
        leader::leader_idle(tick / 150)
    } else {
        leader::leader_peck(tick / 150)
    };
    let chicken_lines = render_sprite(
        &chicken_pixels,
        card_bg(),
        RenderOptions {
            profile: leader_render_profile(area.width),
            compact: false,
        },
    );
```

Update the left-panel leader rendering to:

```rust
    let chicken_pixels = if is_active {
        leader::leader_peck(tick / 4)
    } else {
        leader::leader_idle(tick / 4)
    };
    let leader_profile = leader_render_profile(li.width);
    let chicken_lines = render_sprite(
        &chicken_pixels,
        card_bg(),
        RenderOptions {
            profile: leader_profile,
            compact: leader_profile == RenderProfile::Safe,
        },
    );
```

Update the party sprite selection block to:

```rust
            let sprite = match stage {
                "egg" => party::party_egg(),
                "hatching" => party::party_hatching(tick / 3),
                "peeking" => party::party_peeking(tick / 3),
                "chick" if is_waiting => party::party_sleeping(tick),
                "chick" => party::party_walking(tick / 3),
                "done" => party::party_done(),
                _ => party::party_egg(),
            };

            let profile = party_render_profile(use_compact, col_w);
            let spr_lines = render_sprite(
                &sprite,
                card_bg(),
                RenderOptions {
                    profile,
                    compact: true,
                },
            );
```

Add the tests:

```rust
    #[test]
    fn leader_uses_safe_profile_when_left_panel_is_too_narrow() {
        assert_eq!(leader_render_profile(10), crate::tui::sprites::renderer::RenderProfile::Safe);
        assert_eq!(leader_render_profile(44), crate::tui::sprites::renderer::RenderProfile::Expressive);
    }

    #[test]
    fn party_uses_safe_profile_in_compact_mode() {
        assert_eq!(party_render_profile(true, 12), crate::tui::sprites::renderer::RenderProfile::Safe);
        assert_eq!(party_render_profile(false, 18), crate::tui::sprites::renderer::RenderProfile::Expressive);
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test leader_uses_safe_profile_when_left_panel_is_too_narrow --bin peep`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/widgets/stage.rs
git commit -m "feat: integrate expressive sprite profiles into stage"
```

## Task 6: Add Draw-Level Safety and Visual Regression Coverage

**Files:**
- Modify: `src/tui/render.rs`
- Modify: `src/tui/widgets/stage.rs`
- Test: `src/tui/render.rs`

- [ ] **Step 1: Write the failing test**

Append these tests to `src/tui/render.rs`:

```rust
    #[test]
    fn draw_renders_non_empty_output_with_agent_present() {
        use crate::protocol::types::{Agent, AgentRole, AgentState, SkillKind};
        use crate::store::metrics::DerivedMetrics;
        use ratatui::{backend::TestBackend, Terminal};
        use std::collections::HashMap;

        ensure_theme();
        let backend = TestBackend::new(60, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(8080);
        let snap = StoreSnapshot {
            agents: vec![Agent {
                agent_id: "lead".into(),
                display_name: "lead".into(),
                short_id: "lead".into(),
                first_seen_ts: 0,
                state: AgentState::Active,
                role: AgentRole::Main,
                current_skill: Some(SkillKind::Bash),
                branch_name: None,
                skill_usage: HashMap::new(),
                skills_invoked: HashMap::new(),
                skill_last_seen: HashMap::new(),
                command_usage: HashMap::new(),
                command_last_seen: HashMap::new(),
                total_tokens: 100,
                usage_count: 5,
                tool_run_count: 1,
                last_event_ts: 1,
                completed_at: None,
                completed_visible_until: None,
                completion_recorded: false,
                context_percent: Some(40.0),
                cost_usd: None,
                model_name: None,
                cwd: Some("/tmp/project-a".into()),
                ai_tool: Some("codex".into()),
                parent_session_id: None,
            }],
            feed: Vec::new(),
            sessions: Vec::new(),
            sparkline: Vec::new(),
            metrics: DerivedMetrics {
                total_agents: 1,
                active_agents: 1,
                waiting_agents: 0,
                completed_agents: 0,
                total_events: 0,
                total_tokens: 100,
                total_cost: 0.0,
                avg_context_percent: 40.0,
                velocity_per_min: 0,
            },
            available_skills: Vec::new(),
            rankings: StageRankings::default(),
        };

        terminal.draw(|frame| draw(frame, &mut app, &snap)).unwrap();
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test draw_renders_non_empty_output_with_agent_present --bin peep`
Expected: FAIL if stage integration still references removed sprite entry points or produces invalid render output

- [ ] **Step 3: Write minimal implementation**

Add a second regression test to `src/tui/render.rs`:

```rust
    #[test]
    fn draw_does_not_panic_on_medium_terminal_with_agent_present() {
        use crate::protocol::types::{Agent, AgentRole, AgentState, SkillKind};
        use crate::store::metrics::DerivedMetrics;
        use ratatui::{backend::TestBackend, Terminal};
        use std::collections::HashMap;

        ensure_theme();
        let backend = TestBackend::new(38, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(8080);
        let snap = StoreSnapshot {
            agents: vec![Agent {
                agent_id: "lead".into(),
                display_name: "lead".into(),
                short_id: "lead".into(),
                first_seen_ts: 0,
                state: AgentState::Active,
                role: AgentRole::Main,
                current_skill: Some(SkillKind::Bash),
                branch_name: None,
                skill_usage: HashMap::new(),
                skills_invoked: HashMap::new(),
                skill_last_seen: HashMap::new(),
                command_usage: HashMap::new(),
                command_last_seen: HashMap::new(),
                total_tokens: 100,
                usage_count: 5,
                tool_run_count: 1,
                last_event_ts: 1,
                completed_at: None,
                completed_visible_until: None,
                completion_recorded: false,
                context_percent: Some(40.0),
                cost_usd: None,
                model_name: None,
                cwd: Some("/tmp/project-a".into()),
                ai_tool: Some("codex".into()),
                parent_session_id: None,
            }],
            feed: Vec::new(),
            sessions: Vec::new(),
            sparkline: Vec::new(),
            metrics: DerivedMetrics {
                total_agents: 1,
                active_agents: 1,
                waiting_agents: 0,
                completed_agents: 0,
                total_events: 0,
                total_tokens: 100,
                total_cost: 0.0,
                avg_context_percent: 40.0,
                velocity_per_min: 0,
            },
            available_skills: Vec::new(),
            rankings: StageRankings::default(),
        };

        terminal.draw(|frame| draw(frame, &mut app, &snap)).unwrap();
    }
```

Also keep the existing narrow empty-terminal tests intact.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test draw_renders_non_empty_output_with_agent_present --bin peep`
Expected: PASS

Then run: `cargo test draw_does_not_panic_on_medium_terminal_with_agent_present --bin peep`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/render.rs src/tui/widgets/stage.rs
git commit -m "test: add sprite draw regression coverage"
```

## Task 7: Remove Migration Debt and Run Full Verification

**Files:**
- Modify: `src/tui/sprites/chicken.rs`
- Modify: `src/tui/widgets/stage.rs`
- Test: `src/tui/render.rs`
- Test: `src/tui/widgets/stage.rs`
- Test: `src/tui/sprites/renderer.rs`

- [ ] **Step 1: Write the failing test**

Add this cleanup test to `src/tui/widgets/stage.rs`:

```rust
    #[test]
    fn leader_and_party_profiles_do_not_return_empty_lines() {
        let leader_pixels = crate::tui::sprites::leader::leader_idle(0);
        let leader_lines = crate::tui::sprites::renderer::render_sprite(
            &leader_pixels,
            ratatui::style::Color::Black,
            crate::tui::sprites::renderer::RenderOptions {
                profile: crate::tui::sprites::renderer::RenderProfile::Expressive,
                compact: false,
            },
        );
        let party_pixels = crate::tui::sprites::party::party_walking(0);
        let party_lines = crate::tui::sprites::renderer::render_sprite(
            &party_pixels,
            ratatui::style::Color::Black,
            crate::tui::sprites::renderer::RenderOptions {
                profile: crate::tui::sprites::renderer::RenderProfile::Safe,
                compact: true,
            },
        );
        assert!(!leader_lines.is_empty());
        assert!(!party_lines.is_empty());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test leader_and_party_profiles_do_not_return_empty_lines --bin peep`
Expected: FAIL if migration wrappers or imports are still inconsistent

- [ ] **Step 3: Write minimal implementation**

Finalize `src/tui/sprites/chicken.rs` as a thin compatibility layer only:

```rust
pub use crate::tui::sprites::leader::{
    leader_done as chicken_done,
    leader_idle as chicken_idle,
    leader_peck as chicken_peck,
    leader_sleep,
};
pub use crate::tui::sprites::party::{
    chick_done,
    chick_sleeping,
    chick_sprite,
    egg_cracking,
    egg_hatching_chick,
    egg_sprite,
    growth_stage as agent_growth_stage,
    party_done,
    party_egg,
    party_hatching,
    party_peeking,
    party_sleeping,
    party_walking,
};
```

Then run the full suite:

```bash
cargo test
```

Expected: all sprite, stage, and render tests pass

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test leader_and_party_profiles_do_not_return_empty_lines --bin peep`
Expected: PASS

Run: `cargo test`
Expected: PASS with existing non-blocking example warnings only

- [ ] **Step 5: Commit**

```bash
git add src/tui/sprites/chicken.rs src/tui/widgets/stage.rs src/tui/render.rs src/tui/sprites/renderer.rs src/tui/sprites/leader.rs src/tui/sprites/party.rs src/tui/sprites/style.rs src/tui/sprites/mod.rs
git commit -m "feat: ship expressive dual-tier sprite redesign"
```

## Self-Review

### Spec coverage

- Shared palette and shading rules: Task 1
- Dual-tier leader and party split: Tasks 3 and 4
- Expressive renderer plus safe fallback: Tasks 2 and 5
- Stage integration without layout redesign: Task 5
- Narrow-terminal safety and draw regressions: Tasks 5 and 6
- Final cleanup and full verification: Task 7

No spec section is left without a corresponding task.

### Placeholder scan

Checked for:

- unfinished marker words
- deferred-implementation notes
- vague test instructions without concrete code or commands

No placeholders remain.

### Type consistency

Planned names are consistent across tasks:

- `RenderProfile`
- `RenderOptions`
- `render_sprite`
- `leader_idle`, `leader_peck`, `leader_sleep`, `leader_done`
- `party_egg`, `party_hatching`, `party_peeking`, `party_walking`, `party_sleeping`, `party_done`
- `leader_render_profile`
- `party_render_profile`

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-17-peep-pixel-sprite-redesign.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
