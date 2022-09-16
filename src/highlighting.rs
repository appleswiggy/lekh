use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::as_24_bit_terminal_escaped;
use syntect::easy::HighlightFile;
use std::io::BufRead;

pub struct Highlight {
    highlighter: HighlightFile
}

impl Highlight {
    pub fn default(filename: &str) -> Self {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let mut highlighter: HighlightFile = HighlightFile::new(filename, &ss, &ts.themes["base16-ocean.dark"]).unwrap();

        Self {
            highlighter
        }
    }
}