use ratatui::layout::{Constraint, Layout};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

use crate::FileManager;

impl Widget for &mut FileManager {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Gravily File Manager ".bold());
        let instructions = Line::from(vec![
            " Arrow keys ".into(),
            "<Move>".blue().bold(),
            " Enter ".into(),
            "<Enter Dir>".blue().bold(),
            " Quit ".into(),
            "<q or esc> ".blue().bold(),
        ]);
        let main_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::PLAIN);

        let inner_area = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_block.inner(area));

        main_block.render(area, buf);

        self.render_file_items(inner_area[0], buf);
        self.render_peekable_items(inner_area[1], buf);
    }
}
