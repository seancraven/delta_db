pub mod matcher;
pub mod tui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{self, Alignment, Constraint, Direction, Layout},
    style::Stylize,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Padding, Paragraph, Widget,
    },
    Frame,
};
use std::io::{self, Result};
use tui::restore;
fn main() -> Result<()> {
    let mut t = tui::init()?;
    let mut a = App::new();
    a.run(&mut t)?;
    restore()?;
    Ok(())
}

#[derive(Debug)]
struct App {
    input_box: InputTextbox,
    search_results: Resutlts,
    layout: Layout,
}
impl App {
    pub fn new() -> App {
        let layout = layout::Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(100), Constraint::Min(3)],
        );
        let input_box = InputTextbox::default();
        let search_results = Resutlts {
            r: vec!["Test".into(), "Testy".into()],
        };
        App {
            layout,
            search_results,
            input_box,
        }
    }
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.input_box.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        let l = self.layout.split(frame.size());
        frame.render_widget(&self.search_results, l[0]);
        frame.render_widget(&self.input_box, l[1]);
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
            KeyCode::Char(c) => self.input_box.string.push(c),
            KeyCode::Backspace => {
                self.input_box.string.pop();
            }
            KeyCode::Esc => {
                self.input_box.exit = true;
            }
            _ => (),
        }
    }
}
#[derive(Default, Debug)]
struct InputTextbox {
    string: String,
    exit: bool,
}

impl Widget for &InputTextbox {
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
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_type(ratatui::widgets::BorderType::Rounded);
        Paragraph::new(format!("❯  {}", &*self.string))
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
#[derive(Debug, Default)]
struct Resutlts {
    r: Vec<String>,
}
impl Widget for &Resutlts {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let text_block = Text::from_iter(self.r.iter().map(|line| Line::from(line.as_str())))
            .alignment(Alignment::Left);
        let y_off = area.height - text_block.height() as u16;
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .padding(Padding::new(0, 0, y_off - 1, 0));
        Paragraph::new(text_block).block(block).render(area, buf);
    }
}
