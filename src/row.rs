use std::cmp;
use std::io::stdout;

use crate::SearchDirection;

use crossterm::{
    queue,
    style::{Attribute, SetAttribute},
};
use unicode_segmentation::UnicodeSegmentation;

pub struct Row {
    string: String,
    highlighted: String,
    len: usize,
}

impl Row {
    pub fn default() -> Self {
        Self {
            string: String::new(),
            highlighted: String::new(),
            len: 0,
        }
    }

    pub fn from(st: &str, highlighted: &str) -> Self {
        Self {
            string: String::from(st),
            highlighted: String::from(highlighted),
            len: st.graphemes(true).count(),
        }
    }

    pub fn render(&self, start: usize, end: usize, search_keyword: &Option<String>) {
        let mut prev_esc_seq = String::new();

        let reverse_colors_start: usize;
        let reverse_colors_end: usize;

        if let Some(st) = search_keyword {
            if let Some(pos) = self.find(&st[..], 0, SearchDirection::Forward) {
                reverse_colors_start = pos.saturating_sub(start);
                reverse_colors_end = pos.saturating_add(st.len()).saturating_sub(start);
            }
            else {
                reverse_colors_start = 0;
                reverse_colors_end = 0;
            }
        }
        else {
            reverse_colors_start = 0;
            reverse_colors_end = 0;
        }

        let end = cmp::min(end, self.len);
        let start = cmp::min(start, end);

        let mut flag = false;

        let mut skip = 0;
        let mut chars = 0;

        let mut stdout = stdout();

        for grapheme in self.highlighted[..].graphemes(true) {
            if grapheme == "\x1B" {
                flag = true;
            }
            if flag == true && (grapheme == "m") {
                flag = false;
                prev_esc_seq.push_str(grapheme);
                print!("{}", grapheme);
                continue;
            }

            if flag == false {
                if skip == start {
                    if chars < end - start {
                        if reverse_colors_start + reverse_colors_end != 0 {
                            if chars == reverse_colors_start {
                                if let Err(_) = queue!(stdout, SetAttribute(Attribute::Reverse)) {
                                    panic!("Couldn't write to stdout.");
                                };
                            }
                        }
                        if grapheme == "\t" {
                            print!(" ");
                        } else {
                            print!("{}", grapheme);
                        }
                        chars += 1;

                        if reverse_colors_start + reverse_colors_end != 0 {
                            if chars == reverse_colors_end {
                                if let Err(_) = queue!(stdout, SetAttribute(Attribute::Reset)) {
                                    panic!("Couldn't write to stdout.");
                                };
                            }
                            print!("{}", prev_esc_seq);
                        }
                    } else {
                        break;
                    }
                } else {
                    skip += 1;
                }
            } else {
                prev_esc_seq.push_str(grapheme);
                print!("{}", grapheme);
            }

        }
        if let Err(_) = queue!(stdout, SetAttribute(Attribute::Reset)) {
            panic!("Couldn't write to stdout.");
        };

        print!("{}", prev_esc_seq);
        print!("\r\n");
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get_string(&self) -> &str {
        &self.string[..]
    }

    pub fn get_highlighted(&self) -> &str {
        &self.highlighted[..]
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut result = String::new();
        let mut length = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }

        self.len = length;
        self.string = result;
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len {
            return;
        }

        let mut result = String::new();
        let mut length = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }

        self.len = length;
        self.string = result;
    }

    pub fn append(&mut self, next_row: &Self) {
        self.string = format!("{}{}", self.string, next_row.string);
        self.len += next_row.len;
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row = String::new();
        let mut length = 0;
        let mut splitted_row = String::new();
        let mut splitted_length = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;

        Self {
            string: splitted_row,
            highlighted: self.highlighted.clone(),
            len: splitted_length,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len {
            return None;
        }

        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            at
        };

        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };

        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                substring[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    return Some(start + grapheme_index);
                }
            }
        }

        None
    }
}
