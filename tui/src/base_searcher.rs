use crate::matcher::Matcher;
use crate::tui::Tui;
use crate::{ExitStatus, InputTextbox, Results};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Modifier;
use ratatui::widgets::{Block, BorderType, Paragraph, Widget};
use ratatui::Frame;
use serde_json::Value;
use std::collections::HashMap;
use std::io;

pub struct BaseSearch {
    configs: HashMap<(String, usize), String>,
    input_box: InputTextbox,
    results: Results,
    yaml_display: YamlDisplay,
    layout_outer: Layout,
    layout_inner: Layout,
    matcher: Matcher,
}
impl BaseSearch {
    pub fn new(joint: Vec<(String, usize, Value)>) -> BaseSearch {
        let mut configs = HashMap::new();

        let mut isv = vec![];
        for (name, ver, json) in joint.into_iter() {
            configs.insert((name.clone(), ver), serde_yaml::to_string(&json).unwrap());
            isv.push(format!("{}:{}", name, ver));
        }
        let input_box = InputTextbox {
            string: String::from(""),
            exit: ExitStatus::NoExit,
        };
        let results = Results {
            r: vec![],
            column_matches: vec![],
        };
        let layout = Layout::new(
            Direction::Horizontal,
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        );
        let layout_inner = Layout::new(
            Direction::Vertical,
            vec![Constraint::Percentage(100), Constraint::Min(3)],
        );
        let yaml_display = YamlDisplay(String::from(""), String::from(""));
        let mut matcher = Matcher::new();
        matcher.add_new_strings(isv);
        BaseSearch {
            configs,
            input_box,
            yaml_display,
            results,
            layout_outer: layout,
            layout_inner,
            matcher,
        }
    }
    pub fn get_search_results(&self) -> (String, usize) {
        let Some(res) = self.results.r.last() else {
            return (String::from(""), 0);
        };
        let version = res.chars().next_back().unwrap_or('0');
        let base_name = String::from(&res[..res.len() - 2]);
        (base_name, version.to_string().parse::<usize>().unwrap())
    }
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while self.input_box.exit == ExitStatus::NoExit {
            self.matcher.tick();
            let mut string_results = vec![];
            let mut columnt_results = vec![];
            for (string, column_) in self.matcher.get_matches() {
                string_results.push(string);
                columnt_results.push(column_);
            }
            self.results.r = string_results;
            self.results.column_matches = columnt_results;
            let search = self.get_search_results();
            self.yaml_display.1 = self.configs.get(&search).cloned().unwrap_or_default();
            self.yaml_display.0 = format!("{}:{}", &search.0, &search.1);
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
            self.matcher.add_target(self.input_box.string.as_str());
        }
        Ok(())
    }
    fn render_frame(&self, frame: &mut Frame) {
        let l = self.layout_outer.split(frame.size());
        let l_inner = self.layout_inner.split(l[0]);
        frame.render_widget(&self.results, l_inner[0]);
        frame.render_widget(&self.input_box, l_inner[1]);
        frame.render_widget(&self.yaml_display, l[1]);
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
struct YamlDisplay(String, String);

impl Widget for &YamlDisplay {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!("Base Config: {}", self.0))
            .title_style(Modifier::BOLD)
            .title_alignment(Alignment::Center);

        Paragraph::new(&*self.1).block(block).render(area, buf);
    }
}
