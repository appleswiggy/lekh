use std::error::Error;
use std::process;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::Row;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    filename: Option<String>,
    pub plain_text_colors: String,
}

impl Highlighter {
    pub fn default() -> Self {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let syntax = ss.find_syntax_plain_text();
        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
        let mut plain_text_colors: String = String::new();

        for line in LinesWithEndings::from(" ") {
            let ranges: Vec<(Style, &str)> = if let Ok(ranges) = h.highlight_line(line, &ss) {
                ranges
            } else {
                eprintln!("Error: Couldn't highlight the file.\r");
                process::exit(103);
            };

            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            plain_text_colors = escaped.trim_end().to_string();
            break;
        }

        Self {
            syntax_set: ss,
            theme_set: ts,
            filename: None,
            plain_text_colors,
        }
    }

    pub fn set_file_name(&mut self, filename: String) {
        self.filename = Some(filename);
    }

    pub fn highlight_contents(&self, contents: &str) -> Result<Vec<Row>, Box<dyn Error>> {
        let syntax = match &self.filename {
            Some(file) => self
                .syntax_set
                .find_syntax_for_file(file)?
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text()),
            None => self.syntax_set.find_syntax_plain_text(),
        };

        let mut h = HighlightLines::new(syntax, &self.theme_set.themes["base16-ocean.dark"]);

        let mut res: Vec<Row> = Vec::new();
        for line in LinesWithEndings::from(contents) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &self.syntax_set)?;
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);

            res.push(Row::from(
                &line[..line.len() - 1],
                &escaped[..escaped.len() - 1],
            ));
        }

        Ok(res)
    }
}
