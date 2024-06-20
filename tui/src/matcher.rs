use std::sync::Arc;

use nucleo::{
    self,
    pattern::{CaseMatching, MultiPattern, Normalization, Pattern},
    Config, Nucleo, Utf32String,
};

pub struct Matcher {
    engine: Nucleo<String>,
}
impl Matcher {
    pub fn new() -> Matcher {
        Matcher {
            engine: Nucleo::new(Config::DEFAULT, Arc::new(|| {}), Some(2), 1),
        }
    }
    pub fn add_target(&mut self, target: impl AsRef<str>) {
        self.engine.pattern.reparse(
            0,
            target.as_ref(),
            CaseMatching::Smart,
            Normalization::Smart,
            false,
        )
    }
    pub fn get_target(&self) -> &MultiPattern {
        &self.engine.pattern
    }

    pub fn add_new_strings(&mut self, strings: Vec<String>) {
        let injector = self.engine.injector();
        for string in strings {
            injector.push(string.clone(), |s, asci_str| {
                asci_str[0] = Utf32String::Ascii(s.clone().into_boxed_str());
            });
        }
    }
    pub fn tick(&mut self) {
        self.engine.tick(10);
    }
    pub fn get_matches(&self) -> Vec<(String, Vec<u32>)> {
        let max = self.engine.snapshot().matched_item_count();
        let mut m = nucleo::Matcher::new(Config::DEFAULT);
        let indexer = self.engine.snapshot().pattern().column_pattern(0);
        self.engine
            .snapshot()
            .matched_items(0..max)
            .map(|f| {
                (
                    f.data.clone(),
                    get_indeceis(f.matcher_columns[0].clone(), &mut m, indexer),
                )
            })
            .rev()
            .collect::<Vec<(String, Vec<u32>)>>()
    }
}
fn get_indeceis(a: Utf32String, m: &mut nucleo::Matcher, i: &Pattern) -> Vec<u32> {
    let mut out = vec![];
    i.indices(a.slice(..), m, &mut out);
    out.sort_unstable();
    out.dedup();
    out
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_match_all() {
        let mut m = Matcher::new();
        let t: Vec<String> = vec!["test", "tester", "testest"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        m.add_target("tes");
        m.add_new_strings(t.clone());
        m.tick();
        let matches = m.get_matches();
        for match_ in matches {
            assert!(t.contains(&match_.0))
        }
    }
    #[test]
    fn test_match_one() {
        let mut m = Matcher::new();
        let t: Vec<String> = vec!["test", "tester", "testest"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        m.add_target("tester");
        m.add_new_strings(t.clone());
        m.tick();
        assert_eq!(m.get_matches()[0].0, t[1]);
    }
    #[test]
    fn test_find_none() {
        let mut m = Matcher::new();
        let t: Vec<String> = vec!["test", "tester", "testest"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        m.add_target("dave");
        m.add_new_strings(t.clone());
        m.tick();
        assert!(m.get_matches().len() == 0, "{:?}", m.get_matches().len());
    }
}
