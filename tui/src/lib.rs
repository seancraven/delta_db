mod matcher;
pub mod tui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use matcher::Matcher;
use ratatui::{
    layout::{self, Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Padding, Paragraph, Widget,
    },
    Frame,
};
use serde_json::Value;
use std::{collections::HashMap, io};
pub struct App {
    input_box: InputTextbox,
    search_results: Resutlts,
    display_base: DisplayBox,
    display_delta: DisplayBox,
    outer_layout: Layout,
    inner_layout: Layout,
    matcher: Matcher,
    configs: HashMap<String, String>,
}
impl App {
    pub fn get_search_result(&self) -> String {
        self.configs
            .get(self.search_results.r.last().unwrap_or(&String::from("")))
            .unwrap_or(&String::from(""))
            .clone()
    }
    pub fn new(deltas: Vec<(Value, Value)>, base_cfg: Value) -> App {
        let layout = layout::Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
                Constraint::Min(3),
            ],
        );
        let inner_layout = layout::Layout::new(
            Direction::Horizontal,
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        );
        let input_box = InputTextbox::default();
        let search_results = Resutlts {
            r: vec![],
            column_matches: vec![],
        };
        let mut configs = HashMap::new();
        let mut initial_search_values = Vec::new();
        let base_cfg = serde_yaml::to_string(&base_cfg).unwrap();
        for (delta, full) in deltas {
            let delta_string = serde_json::to_string(&delta).unwrap();
            let full_string = serde_yaml::to_string(&full).unwrap();
            initial_search_values.push(delta_string.clone());
            configs.insert(delta_string, full_string);
        }
        let mut matcher = Matcher::new();
        matcher.add_new_strings(initial_search_values);
        App {
            outer_layout: layout,
            search_results,
            inner_layout,
            display_base: DisplayBox {
                cfg_string: base_cfg,
            },
            display_delta: DisplayBox {
                cfg_string: String::from(""),
            },
            input_box,
            matcher,
            configs,
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
            let delta = self
                .configs
                .get(self.search_results.r.last().unwrap_or(&String::from("")))
                .unwrap();
            self.display_delta.cfg_string = delta.clone();
            self.search_results.column_matches = columnt_results;
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
            self.matcher.add_target(self.input_box.string.as_str());
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        let l = self.outer_layout.split(frame.size());
        let inner_l = self.inner_layout.split(l[0]);
        frame.render_widget(&self.display_base, inner_l[0]);
        frame.render_widget(&self.display_delta, inner_l[1]);
        frame.render_widget(&self.search_results, l[1]);
        frame.render_widget(&self.input_box, l[2]);
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
            KeyCode::Enter => {
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
#[derive(Debug, Default)]
struct DisplayBox {
    cfg_string: String,
}
impl Widget for &DisplayBox {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let base_block = Block::bordered().border_type(BorderType::Rounded);
        Paragraph::new(self.cfg_string.as_str())
            .block(base_block)
            .render(area, buf);
    }
}
