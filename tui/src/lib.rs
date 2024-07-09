pub mod base_searcher;
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
use similar::ChangeTag;
use std::{collections::HashMap, io};
pub struct App {
    input_box: InputTextbox,
    search_results: Results,
    display_base: DisplayBox,
    display_delta: DisplayBox,
    outer_layout: Layout,
    inner_layout: Layout,
    matcher: Matcher,
    configs: HashMap<String, (String, Vec<usize>)>,
}
impl App {
    fn exit_status(&self) -> &ExitStatus {
        &self.input_box.exit
    }
    fn get_best_result(&self) -> String {
        match self.search_results.r.last() {
            Some(res) => self
                .configs
                .get(res)
                .cloned()
                .map(|x| x.0)
                .unwrap_or(String::from("")),
            None => String::from(""),
        }
    }
    fn get_result_to_hightlight(&self) -> Option<(String, Vec<usize>)> {
        let res = self.search_results.r.last()?;
        self.configs.get(res).cloned()
    }
    pub fn get_search_result(&self) -> String {
        match self.exit_status() {
            ExitStatus::Quit => String::from(""),
            ExitStatus::Finish => self.get_best_result(),
            ExitStatus::NoExit => String::from(""),
        }
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
        let search_results = Results {
            r: vec![],
            column_matches: vec![],
        };
        let mut configs = HashMap::new();
        let mut initial_search_values = Vec::new();
        let base_cfg = serde_yaml::to_string(&base_cfg).unwrap();

        for (delta, full) in deltas {
            let delta_string = serde_json::to_string(&delta).unwrap();
            let full_string = serde_yaml::to_string(&full).unwrap();
            let diff_line = diff_by_lines(&base_cfg, &full_string);
            initial_search_values.push(delta_string.clone());
            configs.insert(delta_string, (full_string, diff_line));
        }

        let mut matcher = Matcher::new();
        matcher.add_new_strings(initial_search_values);

        App {
            outer_layout: layout,
            inner_layout,
            input_box,
            search_results,
            display_base: DisplayBox {
                cfg_string: base_cfg,
                highlight_lines: Vec::new(),
                title: String::from("Base Config"),
            },
            display_delta: DisplayBox {
                cfg_string: String::from(""),
                highlight_lines: Vec::new(),
                title: String::from("Changed Config"),
            },
            matcher,
            configs,
        }
    }
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while self.input_box.exit == ExitStatus::NoExit {
            self.matcher.tick();
            let mut string_results = vec![];
            let mut columnt_results = vec![];
            for (string, column_) in self.matcher.get_matches() {
                string_results.push(string);
                columnt_results.push(column_);
            }
            self.search_results.r = string_results;
            self.search_results.column_matches = columnt_results;
            let (string, highlighs) = self.get_result_to_hightlight().unwrap_or_default();
            self.display_delta.cfg_string = string;
            self.display_delta.highlight_lines = highlighs;
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
                self.input_box.exit = ExitStatus::Quit;
            }
            KeyCode::Enter => {
                self.input_box.exit = ExitStatus::Finish;
            }
            _ => (),
        }
    }
}
#[derive(Default, Debug, PartialEq)]
enum ExitStatus {
    #[default]
    NoExit,
    Quit,
    Finish,
}
#[derive(Default, Debug)]
struct InputTextbox {
    string: String,
    exit: ExitStatus,
}

impl Widget for &InputTextbox {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Title::from("Search for delta".bold());
        let instructions = Title::from(Line::from(vec!["Quit<Esc> / Return <Enter>".bold()]));
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
struct Results {
    r: Vec<String>,
    column_matches: Vec<Vec<u32>>,
}
impl Widget for &Results {
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
impl Results {
    fn format_line(&self, l: &str, indecies: &[u32]) -> Line {
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
    highlight_lines: Vec<usize>,
    title: String,
}
impl Widget for &DisplayBox {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let base_block = Block::bordered().border_type(BorderType::Rounded).title(
            Title::from(Span::from(&*self.title).style(Color::Green)).alignment(Alignment::Center),
        );
        let mut lines = Vec::new();
        for (i, line) in self.cfg_string.lines().enumerate() {
            if self.highlight_lines.contains(&i) {
                lines.push(Line::from(line).cyan());
            } else {
                lines.push(Line::from(line).white());
            }
        }
        Paragraph::new(Text::from_iter(lines))
            .block(base_block)
            .alignment(Alignment::Left)
            .render(area, buf);
    }
}

fn diff_by_lines(base_string: &str, other: &str) -> Vec<usize> {
    let lines = similar::TextDiff::from_lines(base_string, other);
    let mut out = vec![];
    for change in lines.iter_all_changes() {
        if change.tag() != ChangeTag::Equal {
            if let Some(c) = change.old_index() {
                out.push(c);
            }
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        let line_a = "a\nb\nc\n";
        let line_b = "a\nc\nc\n";
        let out = diff_by_lines(line_a, line_b);
        assert_eq!(out, vec![1]);
    }
}
