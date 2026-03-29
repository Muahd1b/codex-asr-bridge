pub mod bottom;
pub mod middle;
pub mod top;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < 96 || area.height < 28 {
        frame.render_widget(
            Paragraph::new("Terminal too small. Need at least 96x28 for 3-pane layout.").block(
                Block::default()
                    .title("Voxtral Flow Dictation")
                    .borders(Borders::ALL),
            ),
            area,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(10),
            Constraint::Length(8),
        ])
        .split(area);

    top::draw(frame, chunks[0], app);
    middle::draw(frame, chunks[1], app);
    bottom::draw(frame, chunks[2], app);
}
