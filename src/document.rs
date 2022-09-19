use std::fs;
use std::io::Write;

use crate::Highlighter;
use crate::Position;
use crate::Row;
use crate::SearchDirection;

pub struct Document {
    rows: Vec<Row>,
    file_name: Option<String>,
    dirty: bool,
    pub highlighter: Highlighter,
}

impl Document {
    pub fn default() -> Self {
        let highlighter = Highlighter::default();
        Self {
            rows: vec![],
            file_name: None,
            dirty: false,
            highlighter,
        }
    }
   
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;

        let mut highlighter = Highlighter::default();
        highlighter.set_file_name(filename.to_string());

        let rows: Vec<Row> = highlighter.highlight_contents(&contents[..]);

        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
            highlighter,
        })
    }

    pub fn get_file_name(&self) -> Option<String> {
        if let Some(filename) = &self.file_name {
            Some(filename.clone())
        } else {
            None
        }
    }

    pub fn set_file_name(&mut self, file_name: String) {
        self.file_name = Some(file_name);
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn highlight(&mut self) {
        if let Some(filename) = &self.file_name {
            self.highlighter.set_file_name(filename.to_string());
        }
        let highlighter = &self.highlighter;

        let mut contents = String::new();
        for row in &self.rows {
            contents.push_str(row.get_string());
            contents.push('\n');
        }

        let rows: Vec<Row> = highlighter.highlight_contents(&contents[..]);
        self.rows = rows;
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }
        self.dirty = true;

        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let new_row = self.rows.get_mut(at.y).unwrap().split(at.x);
        self.rows.insert(at.y + 1, new_row);

        self.highlight();
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }
        self.dirty = true;

        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }

        self.highlight();
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        if at.y >= len {
            return;
        }
        self.dirty = true;

        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }

        self.highlight();
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }

        let mut position = Position { x: at.x, y: at.y };

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }

                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }
}
