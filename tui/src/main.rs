pub mod tui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Alignment,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Widget,
    },
    Frame,
};
use std::io::{self, Result};
fn main() -> Result<()> {
    let mut t = tui::init()?;
    let mut a = App::default();
    a.run(&mut t)?;
    Ok(())
}

#[derive(Default, Debug)]
struct App {
    string: String,
    exit: bool,
}
impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => self.handle_key_events(k),
            _ => {}
        };
        Ok(())
    }
    fn handle_key_events(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Char(c) => self.string.push(c),
            KeyCode::Backspace => {
                self.string.pop();
            }
            KeyCode::Esc => {
                self.exit = true;
            }
            _ => (),
        }
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Title::from("Search for delta".bold());
        let instructions = Title::from(Line::from(vec!["Quit".bold(), "<Esc>".bold()]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::all())
            .border_set(border::THICK);
        Paragraph::new(&*self.string)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
