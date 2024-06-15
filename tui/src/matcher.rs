use std::sync::Arc;

use nucleo::{self, Config, Nucleo, Utf32String};

pub struct Matcher {
    engine: Nucleo<String>,
}
impl Matcher {
    fn new() -> Matcher {
        Matcher {
            engine: Nucleo::new(Config::DEFAULT, Arc::new(|| {}), Some(2), 1),
        }
    }

    fn add_new_strings(&mut self, strings: Vec<String>) {
        let injector = self.engine.injector();
        for string in strings {
            injector.push(string.clone(), |s, asci_str| {
                asci_str[0] = Utf32String::Ascii(s.clone().into_boxed_str());
            });
        }
    }
    fn get_matches(&self) {
        let it = self
            .engine
            .snapshot()
            .matched_items(0..30)
            .map(|f| f.data.clone())
            .collect::<Vec<String>>();
    }
}
