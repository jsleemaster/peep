use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

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

    #[test]
    fn expressive_renderer_uses_quadrant_glyph_and_majority_color() {
        let red = Some(Color::Rgb(220, 40, 40));
        let blue = Some(Color::Rgb(40, 40, 220));
        let sprite = vec![vec![red, red], vec![blue, None]];

        let lines = render_sprite(
            &sprite,
            Color::Black,
            RenderOptions {
                profile: RenderProfile::Expressive,
                compact: false,
            },
        );

        assert_eq!(lines.len(), 1);
        let span = &lines[0].spans[0];
        assert_eq!(span.content.as_ref(), "▛▛");
        assert_eq!(span.style.fg, red);
        assert_eq!(span.style.bg, Some(Color::Black));
    }

    #[test]
    fn safe_renderer_keeps_half_block_content_and_style() {
        let green = Some(Color::Rgb(40, 180, 90));
        let sprite = vec![vec![green, None], vec![None, None]];

        let lines = render_sprite(
            &sprite,
            Color::Black,
            RenderOptions {
                profile: RenderProfile::Safe,
                compact: false,
            },
        );

        assert_eq!(lines.len(), 1);
        let span = &lines[0].spans[0];
        assert_eq!(span.content.as_ref(), "▀▀");
        assert_eq!(span.style.fg, green);
        assert_eq!(span.style.bg, Some(Color::Black));
    }
}

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

/// Convert a sprite (2D grid of Option<Color>) to terminal lines using half-block rendering.
/// Each pixel is rendered as 2 horizontal characters for square aspect ratio.
pub fn sprite_to_lines(pixels: &[Vec<Option<Color>>], bg: Color) -> Vec<Line<'static>> {
    render_sprite(
        pixels,
        bg,
        RenderOptions {
            profile: RenderProfile::Safe,
            compact: false,
        },
    )
}

pub fn sprite_to_lines_compact(pixels: &[Vec<Option<Color>>], bg: Color) -> Vec<Line<'static>> {
    render_sprite(
        pixels,
        bg,
        RenderOptions {
            profile: RenderProfile::Safe,
            compact: true,
        },
    )
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
                sprite_to_lines_compact_impl(pixels, bg)
            } else {
                sprite_to_lines_impl(pixels, bg)
            }
        }
    }
}

fn sprite_to_lines_impl(pixels: &[Vec<Option<Color>>], bg: Color) -> Vec<Line<'static>> {
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

fn sprite_to_lines_compact_impl(pixels: &[Vec<Option<Color>>], bg: Color) -> Vec<Line<'static>> {
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
            let mask = (ul.is_some() as u8)
                | ((ur.is_some() as u8) << 1)
                | ((ll.is_some() as u8) << 2)
                | ((lr.is_some() as u8) << 3);
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
    let mut counts: Vec<(Color, usize, usize)> = Vec::new();
    for (idx, color) in colors.into_iter().enumerate() {
        let Some(color) = color else {
            continue;
        };

        if let Some((_, count, _)) = counts.iter_mut().find(|(existing, _, _)| *existing == color)
        {
            *count += 1;
        } else {
            counts.push((color, 1, idx));
        }
    }

    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2)))
        .map(|(color, _, _)| color)
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
