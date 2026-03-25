use ratatui::style::Color;

const EYE_WHITE: Color = Color::Rgb(240, 240, 255);
const EYE_PUPIL: Color = Color::Rgb(20, 20, 60);

/// Color palette for agents (body, dark).
const PALETTE: [(Color, Color); 8] = [
    (Color::Rgb(255, 220, 50), Color::Rgb(200, 170, 30)),   // yellow (classic)
    (Color::Rgb(80, 200, 255), Color::Rgb(50, 150, 200)),   // cyan
    (Color::Rgb(255, 140, 200), Color::Rgb(200, 100, 160)), // pink
    (Color::Rgb(180, 100, 255), Color::Rgb(130, 60, 200)),  // purple
    (Color::Rgb(255, 160, 50), Color::Rgb(200, 120, 30)),   // orange
    (Color::Rgb(100, 255, 100), Color::Rgb(60, 200, 60)),   // lime
    (Color::Rgb(255, 100, 100), Color::Rgb(200, 60, 60)),   // red
    (Color::Rgb(255, 255, 150), Color::Rgb(200, 200, 100)), // cream
];

/// 5x6 mini Pac-Man with mouth open (facing right).
/// 0=transparent, 1=body, 2=dark, 3=eye_white, 4=eye_pupil
pub fn mini_packman_open() -> Vec<Vec<u8>> {
    vec![
        vec![0, 1, 1, 1, 0], // row 0
        vec![1, 3, 4, 1, 0], // row 1 (eye)
        vec![1, 1, 1, 0, 0], // row 2 (mouth open)
        vec![1, 1, 1, 0, 0], // row 3
        vec![1, 1, 1, 1, 0], // row 4
        vec![0, 1, 1, 1, 0], // row 5
    ]
}

/// 5x6 mini Pac-Man with mouth closed.
pub fn mini_packman_closed() -> Vec<Vec<u8>> {
    vec![
        vec![0, 1, 1, 1, 0],
        vec![1, 3, 4, 1, 1],
        vec![1, 1, 1, 1, 1],
        vec![1, 2, 2, 2, 1],
        vec![1, 1, 1, 1, 1],
        vec![0, 1, 1, 1, 0],
    ]
}

/// 5x6 mini ghost shape.
pub fn mini_ghost() -> Vec<Vec<u8>> {
    vec![
        vec![0, 1, 1, 1, 0],
        vec![1, 3, 4, 3, 4],
        vec![1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1],
        vec![1, 0, 1, 0, 1],
    ]
}

/// 5x6 done state (circle with checkmark).
pub fn mini_done() -> Vec<Vec<u8>> {
    vec![
        vec![0, 1, 1, 1, 0],
        vec![1, 1, 1, 3, 1],
        vec![1, 1, 3, 1, 1],
        vec![3, 1, 1, 1, 1],
        vec![1, 3, 1, 1, 1],
        vec![0, 1, 1, 1, 0],
    ]
}

/// Colorize a template with body and dark body colors.
pub fn colorize(
    template: &[Vec<u8>],
    body: Color,
    dark: Color,
) -> Vec<Vec<Option<Color>>> {
    template
        .iter()
        .map(|row| {
            row.iter()
                .map(|&v| match v {
                    0 => None,
                    1 => Some(body),
                    2 => Some(dark),
                    3 => Some(EYE_WHITE),
                    4 => Some(EYE_PUPIL),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

/// Flip a sprite horizontally.
pub fn flip_h(sprite: &[Vec<Option<Color>>]) -> Vec<Vec<Option<Color>>> {
    sprite
        .iter()
        .map(|row| {
            let mut r = row.clone();
            r.reverse();
            r
        })
        .collect()
}

/// Deterministic color selection based on agent_id hash.
/// The first palette entry (classic yellow) is index 0.
/// Uses agent_id hash to pick from palette, but spreads across all colors.
pub fn agent_colors(agent_id: &str) -> (Color, Color) {
    let hash: u64 = agent_id
        .bytes()
        .fold(5381u64, |h, b| h.wrapping_mul(33).wrapping_add(b as u64));
    // Start from index 0 so first agents tend to get the first colors
    let idx = (hash as usize) % PALETTE.len();
    PALETTE[idx]
}

/// Get color for agent by index (0 = classic yellow packman).
pub fn agent_colors_by_index(index: usize) -> (Color, Color) {
    PALETTE[index % PALETTE.len()]
}
