use super::FileManager;

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    style::Style,
    style::Color,
    widgets::{Block, List, Paragraph, StatefulWidget, Widget, Wrap},
};

use std::fs::{metadata, read_dir, read_to_string};
use std::path::PathBuf;
use image::{DynamicImage, GenericImageView, imageops::FilterType, Pixel};


pub struct ImageWidget {
    img: DynamicImage,
}

impl ImageWidget {
    pub fn new(img: DynamicImage) -> Self {
        Self { img }
    }
}

impl Widget for ImageWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let resized = self.img.resize_exact(
            area.width as u32,
            area.height as u32 * 2,
            FilterType::Triangle,
        );

        let (img_width, img_height) = resized.dimensions();

        for y in 0..area.height {
            for x in 0..area.width {
                let top_pixel = resized.get_pixel(x as u32, y as u32 * 2).to_rgba();
                let bottom_pixel = if y as u32 * 2 + 1 < img_height {
                    resized.get_pixel(x as u32, y as u32 * 2 + 1).to_rgba()
                } else {
                    top_pixel
                };

                let fg = Color::Rgb(top_pixel[0], top_pixel[1], top_pixel[2]);
                let bg = Color::Rgb(bottom_pixel[0], bottom_pixel[1], bottom_pixel[2]);

                buf.get_mut(area.x + x, area.y + y)
                    .set_style(Style::default().fg(fg).bg(bg))
                    .set_symbol("▀");
            }
        }
    }
}


impl FileManager {
    pub fn render_error_text(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(" Error ('x' to close) ")
            .border_set(border::ROUNDED);

        let text = Line::from(self.error.to_string());

        let error_box = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Left)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(error_box, area)
    }

    pub fn render_confirmation_text(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(path_val) = self.state.selected() {
            let file_name = &self.path_items[path_val];

            let block = Block::bordered()
                .title(" Confirmation ('Y' to Confirm or 'N' to Cancel) ")
                .border_set(border::ROUNDED);

            let text = Line::from(format!("Are you sure you want to delete {:?}?", file_name));

            let error_box = Paragraph::new(text)
                .alignment(ratatui::layout::Alignment::Left)
                .block(block)
                .wrap(Wrap { trim: true });

            frame.render_widget(error_box, area)
        }
    }

    pub fn render_input_text(&mut self, frame: &mut Frame, area: Rect) {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);

        let block = Block::bordered()
            .title(" Input ")
            .border_set(border::ROUNDED);

        let text = Line::from(self.input.to_string());

        let input_box = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Left)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((0, scroll as u16));

        frame.render_widget(input_box, area);
    }

    pub fn render_file_items(&mut self, area: Rect, buf: &mut Buffer) {
        let path_text = Line::from(vec![
            " Current Path: ".into(),
            self.path.to_str().unwrap().to_string().yellow(),
            " ".into(),
        ]);

        let block = Block::bordered()
            .title(path_text)
            .border_set(border::ROUNDED);

        match read_dir(&self.path) {
            Ok(path_items) => {
                self.path_items = path_items
                    .filter_map(|entry_result| {
                        entry_result
                            .ok()
                            .map(|entry| entry.path().file_name().unwrap().to_owned()) // direntry -> pathbuf -> osstr
                    })
                    .collect();
            }

            Err(e) => {
                self.error = format!("Error entering dir: {:#?}: {}", &self.path, e);
                self.exit_dir();
            }
        }

        let items: Vec<String> = self
            .path_items
            .iter()
            .map(|file_name| String::from(file_name.to_str().unwrap()))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">    ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    pub fn render_peekable_items(&mut self, area: Rect, buf: &mut Buffer) {
        let cur_path = self.get_hovered_dir();
        if cur_path != PathBuf::from(&self.path) {
            let block = Block::bordered()
                .title(Line::from(vec![
                    " ".into(),
                    cur_path.to_str().unwrap().to_string().yellow(),
                    " ".into(),
                ]))
                .border_set(border::ROUNDED);

            match metadata(&cur_path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        match read_dir(&cur_path) {
                            Ok(path) => {
                                let items: Vec<String> = path
                                    .filter_map(|entry_result| entry_result.ok())
                                    .filter_map(|entry| {
                                        let path_buf = entry.path();

                                        path_buf
                                            .strip_prefix(&cur_path)
                                            .ok()
                                            .map(|relative_path| {
                                                relative_path.to_str().map(str::to_owned)
                                            })
                                            .flatten()
                                    })
                                    .collect();
                                let list = List::new(items).block(block);
                                Widget::render(list, area, buf);
                            }
                            Err(e) => {
                                Paragraph::new(format!(
                                    "Failed to read directory\n > {}\n\nError: {}",
                                    &cur_path.display(),
                                    e
                                ))
                                .block(block)
                                .render(area, buf);
                            }
                        }
                    } else if metadata.is_file() {
                        if let Ok(file_text) = read_to_string(&cur_path) {
                            Paragraph::new(file_text).block(block).render(area, buf);
                            return;
                        }
                        if let Ok(img) = image::open(&cur_path) {
                            let image_widget = ImageWidget::new(img);
                            image_widget.render(area, buf);

                        }
                        }
                }
                Err(e) => {
                    self.error = format!(
                        "Error getting metadata of path {}: {}",
                        cur_path.display(),
                        e
                    )
                }
            }
        }
    }
}
