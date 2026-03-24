use ratatui::{
    layout::Rect,
    style::Color,
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::App;
use crate::tui::render::StoreSnapshot;
use crate::tui::sprites::renderer::PixelCanvas;

const STAGE_BG: Color = Color::Rgb(15, 15, 25);

pub fn render_stage(f: &mut Frame, area: Rect, app: &App, _snap: &StoreSnapshot) {
    let canvas_w = area.width as usize;
    let canvas_h = (area.height as usize) * 2; // half-block doubling

    if canvas_w == 0 || canvas_h == 0 {
        return;
    }

    let mut canvas = PixelCanvas::new(canvas_w, canvas_h);
    app.stage.render(&mut canvas, app.tick);

    let lines = canvas.to_lines(STAGE_BG);
    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}
