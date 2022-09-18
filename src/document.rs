use std::fs;
use std::io::Write;

use crate::Highlight;
use crate::Position;
use crate::Row;
use crate::SearchDirection;

pub struct Document {
    rows: Vec<Row>,
    highlighted_rows: Vec<Row>,
    file_name: Option<String>,
    dirty: bool,
    pub highlight: Highlight,
}

impl Document {
    pub fn default() -> Self {
        let highlight = Highlight::default();
        Self {
            rows: vec![],
            highlighted_rows: vec![],
            file_name: None,
            dirty: false,
            highlight,
        }
    }
   
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();

        for line in contents.lines() {
            rows.push(Row::from(line));
        }

        let mut highlight = Highlight::default();
        highlight.set_file_name(filename.to_string());

        let highlighted_rows: Vec<Row> = highlight.highlight_contents(&contents[..]);

        assert_eq!(highlighted_rows.len(), rows.len());

        Ok(Self {
            rows,
            highlighted_rows,
            file_name: Some(filename.to_string()),
            dirty: false,
            highlight,
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

    pub fn highlighted_row(&self, index: usize) -> Option<(&Row, usize)> {
        let row = self.rows.get(index);
        let highlighted_row = self.highlighted_rows.get(index);

        if row.is_none() || highlighted_row.is_none() {
            return None;
        }
        
        return Some((highlighted_row.unwrap(), row.unwrap().len()));
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    fn highlight(&mut self) {
        if let Some(filename) = &self.file_name {
            self.highlight.set_file_name(filename.to_string());
        }
        let highlight = &self.highlight;

        let mut contents = String::new();
        for row in &self.rows {
            contents.push_str(row.get_string());
            contents.push('\n');
        }

        let highlighted_rows: Vec<Row> = highlight.highlight_contents(&contents[..]);

        assert_eq!(highlighted_rows.len(), self.rows.len());
        self.highlighted_rows = highlighted_rows;
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
