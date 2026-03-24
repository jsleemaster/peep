use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// A 2D pixel canvas with optional colors per pixel.
/// Height must be even for half-block rendering.
pub struct PixelCanvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<Option<Color>>>,
}

impl PixelCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        // Ensure even height
        let height = if !height.is_multiple_of(2) { height + 1 } else { height };
        let pixels = vec![vec![None; width]; height];
        PixelCanvas {
            width,
            height,
            pixels,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            self.pixels[y][x] = Some(color);
        }
    }

    /// Blit a sprite onto the canvas. None pixels are transparent (skip).
    pub fn blit(&mut self, sprite: &[Vec<Option<Color>>], dx: usize, dy: usize) {
        for (sy, row) in sprite.iter().enumerate() {
            let py = dy + sy;
            if py >= self.height {
                break;
            }
            for (sx, pixel) in row.iter().enumerate() {
                let px = dx + sx;
                if px >= self.width {
                    break;
                }
                if let Some(c) = pixel {
                    self.pixels[py][px] = Some(*c);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for py in y..y + h {
            if py >= self.height {
                break;
            }
            for px in x..x + w {
                if px >= self.width {
                    break;
                }
                self.pixels[py][px] = Some(color);
            }
        }
    }

    /// Convert to terminal lines using half-block rendering.
    /// Processes 2 pixel rows per terminal row.
    pub fn to_lines(&self, bg: Color) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let mut y = 0;
        while y < self.height {
            let mut spans = Vec::new();
            for x in 0..self.width {
                let top = self.pixels[y][x];
                let btm = if y + 1 < self.height {
                    self.pixels[y + 1][x]
                } else {
                    None
                };

                match (top, btm) {
                    (Some(tc), Some(bc)) => {
                        spans.push(Span::styled(
                            "\u{2580}", // upper half block
                            Style::default().fg(tc).bg(bc),
                        ));
                    }
                    (Some(tc), None) => {
                        spans.push(Span::styled(
                            "\u{2580}",
                            Style::default().fg(tc).bg(bg),
                        ));
                    }
                    (None, Some(bc)) => {
                        spans.push(Span::styled(
                            "\u{2584}", // lower half block
                            Style::default().fg(bc).bg(bg),
                        ));
                    }
                    (None, None) => {
                        spans.push(Span::styled(" ", Style::default().bg(bg)));
                    }
                }
            }
            lines.push(Line::from(spans));
            y += 2;
        }
        lines
    }
}
