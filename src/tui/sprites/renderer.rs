use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Convert a sprite (2D grid of Option<Color>) to terminal lines using half-block rendering.
/// Each pixel is rendered as 2 horizontal characters for square aspect ratio.
pub fn sprite_to_lines(pixels: &[Vec<Option<Color>>], bg: Color) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut y = 0;
    while y < pixels.len() {
        let mut spans = Vec::new();
        for x in 0..pixels[y].len() {
            let top = pixels[y][x];
            let btm = if y + 1 < pixels.len() {
                pixels[y + 1][x]
            } else {
                None
            };
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
