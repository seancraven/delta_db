pub mod matcher;
pub mod tui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use matcher::Matcher;
use ratatui::{
    layout::{self, Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Padding, Paragraph, Widget,
    },
    Frame,
};
use std::{
    fs::{read_to_string, File},
    io::{self, Read, Result},
};
use tui::restore;
fn main() -> Result<()> {
    let mut t = tui::init()?;
    let f = read_to_string("src/tui.rs")?
        .lines()
        .map(String::from)
        .collect();
    let mut a = App::new(f);
    a.run(&mut t)?;
    restore()?;
    Ok(())
}

struct App {
    input_box: InputTextbox,
    search_results: Resutlts,
    layout: Layout,
    matcher: Matcher,
}
impl App {
    pub fn new(inital_search_values: Vec<String>) -> App {
        let layout = layout::Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(100), Constraint::Min(3)],
        );
        let input_box = InputTextbox::default();
        let search_results = Resutlts {
            r: vec![],
            column_matches: vec![],
        };
        let mut matcher = Matcher::new();
        matcher.add_new_strings(inital_search_values);
        App {
            layout,
            search_results,
            input_box,
            matcher,
        }
    }
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.input_box.exit {
            self.matcher.tick();
            let mut string_results = vec![];
            let mut columnt_results = vec![];
            for (string, column_) in self.matcher.get_matches() {
                string_results.push(string);
                columnt_results.push(column_);
            }
            self.search_results.r = string_results;
            self.search_results.column_matches = columnt_results;
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
            self.matcher.add_target(self.input_box.string.as_str());
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
        Paragraph::new(Line::from(vec![
            Span::styled("❯  ", Style::default().fg(Color::Green)),
            Span::from(&*self.string),
        ]))
        .left_aligned()
        .block(block)
        .render(area, buf);
    }
}
#[derive(Debug, Default)]
struct Resutlts {
    r: Vec<String>,
    column_matches: Vec<Vec<u32>>,
}
impl Widget for &Resutlts {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let text_block = Text::from_iter(
            self.r
                .iter()
                .zip(self.column_matches.iter())
                .map(|(l, i)| self.format_line(l, i)),
        )
        .alignment(Alignment::Left);
        let y_off = area.height - text_block.height() as u16;
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .padding(Padding::new(0, 0, y_off - 1, 0));
        Paragraph::new(text_block).block(block).render(area, buf);
    }
}
impl Resutlts {
    fn format_line(&self, l: &str, indecies: &Vec<u32>) -> Line {
        Line::from(
            l.char_indices()
                .map(|(i, c)| match indecies.contains(&(i as u32)) {
                    true => Span::styled(c.to_string(), Style::new().fg(Color::Green)),
                    false => Span::raw(c.to_string()),
                })
                .collect::<Vec<Span>>(),
        )
    }
}
