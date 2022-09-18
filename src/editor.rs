#![warn(clippy::all, clippy::pedantic)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;

use std::env;
use std::time::Duration;
use std::time::Instant;

use crate::Document;
use crate::Row;
use crate::Terminal;

const STATUS_FG_COLOR: Color = Color::Black;
const STATUS_BG_COLOR: Color = Color::White;
const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 1;
const TAB_SIZE: u8 = 4;

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            text: message,
            time: Instant::now(),
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
}

impl Editor {
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");

        let document = if let Some(file_name) = args.get(1) {
            if let Ok(doc) = Document::open(&file_name) {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal."),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
        }
    }

    pub fn run(&mut self) {
        if let Err(_) = self.terminal.enter_alternate_screen() {
            panic!("Can't enter alternate screen.");
        }
        Terminal::into_raw_mode();

        loop {
            if self.refresh_screen().is_err() {
                panic!("Can't clear the terminal screen");
            }

            if self.should_quit {
                break;
            }

            if self.process_keypress().is_err() {
                panic!("Can't process keypress.");
            }
        }

        self.terminal.cleanup_and_exit(0);
    }

    fn quit(&mut self) -> Result<(), std::io::Error> {
        let mut quit = true;

        if self.document.is_dirty() {
            let mut result;
            loop {
                result = self.prompt("Save Modified Buffer? (Y or N): ", |_, _, _| {})?;

                if let Some(response) = result {
                    match &*response {
                        "y" | "Y" => {
                            if let Err(_) = self.save() {
                                quit = false;
                            }
                            break;
                        }
                        "n" | "N" => {
                            break;
                        },
                        _ => (),
                    }
                } else {
                    quit = false;
                    break;
                }
            }
        }
        self.should_quit = quit;
        Ok(())
    }

    fn save(&mut self) -> Result<(), &str> {
        if self.document.get_file_name().is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            match new_name {
                Some(file_name) => self.document.set_file_name(file_name),
                None => {
                    self.status_message = StatusMessage::from("Save aborted.".to_string());
                    return Err("Can't save file.");
                }
            }
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
            return Err("Can't save file.");
        }

        return Ok(());
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;

        let mut found = false;

        let query = self
            .prompt(
                "Search (ESC to cancel, Arrows to navigate): ",
                |editor, key, query| {
                    let mut moved = false;
                    found = false;

                    match key {
                        KeyCode::Right | KeyCode::Down => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(KeyCode::Right);
                            moved = true;
                        }
                        KeyCode::Left | KeyCode::Up => {
                            direction = SearchDirection::Backward;
                        }
                        _ => direction = SearchDirection::Forward,
                    }

                    if let Some(position) =
                        editor
                            .document
                            .find(&query, &editor.cursor_position, direction)
                    {
                        editor.cursor_position = position;
                        editor.scroll();
                        found = true;
                    } else if moved {
                        editor.move_cursor(KeyCode::Left);
                    }
                },
            )
            .unwrap_or(None);
        
        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        else if !found {
            self.status_message = StatusMessage::from("No results found.".to_string());
            self.cursor_position = old_position;
            self.scroll();
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let key_event: KeyEvent = Terminal::read_key(|width, height| {
            self.terminal.set_size(width, height);
            self.scroll();
            self.refresh_screen()?;
            Ok(())
        })?;

        match key_event.code {
            KeyCode::Char(ch) => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    match ch {
                        'q' | 'Q' => self.quit()?,
                        's' | 'S' => {
                            if self.save().is_err() {
                                // do nothing
                            };
                        },
                        'f' | 'F' => self.search(),
                        _ => (),
                    }
                } else {
                    self.document.insert(&self.cursor_position, ch);
                    self.move_cursor(KeyCode::Right);
                }
            }
            KeyCode::Delete => self.document.delete(&self.cursor_position),
            KeyCode::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(KeyCode::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            KeyCode::Enter => {
                self.document.insert_newline(&self.cursor_position);
                self.move_cursor(KeyCode::Right);
            }
            KeyCode::Tab => {
                for _ in 0..TAB_SIZE {
                    self.document.insert(&self.cursor_position, ' ');
                    self.move_cursor(KeyCode::Right);
                }
            }
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::PageDown
            | KeyCode::PageUp
            | KeyCode::Home
            | KeyCode::End => {
                self.move_cursor(key_event.code);
            }
            _ => (),
        }

        self.scroll();

        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.get_size().width as usize;
        let height = self.terminal.get_size().height as usize;
        let mut offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide()?;
        self.terminal.move_cursor(&Position::default())?;

        if self.should_quit {
            self.terminal.clear_screen()?;
        } else {
            self.draw_status_bar()?;
            print!("{}", self.document.highlight.plain_text_colors);

            self.draw_rows()?;
            self.draw_message_bar()?;

            self.terminal.move_cursor(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y).saturating_add(1),
            })?;
        }

        self.terminal.cursor_show()?;
        Ok(())
    }

    fn move_cursor(&mut self, key: KeyCode) {
        let terminal_height = self.terminal.get_size().height as usize;
        let Position { mut x, mut y } = self.cursor_position;

        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                };
            }
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y };
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Lekh editor -- version {}", VERSION);

        let width = self.terminal.get_size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);

        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row, len: usize) {
        let width = self.terminal.get_size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);

        row.render(start, end, len);
    }

    fn draw_rows(&mut self) -> Result<(), std::io::Error> {
        let height = self.terminal.get_size().height;
        for terminal_row in 0..height {
            self.terminal.clear_current_line()?;
            if let Some((row, len)) = self
                .document
                .highlighted_row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row, len);

            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                print!("~\r\n");
            }
        }

        if let Err(_) = self.terminal.flush() {
            panic!("Error flushing terminal.");
        }

        Ok(())
    }

    fn draw_status_bar(&mut self) -> Result<(), std::io::Error> {
        let mut status;
        let width = self.terminal.get_size().width as usize;
        let mut file_name = "[No Name]".to_string();

        let modified_indicator = if self.document.is_dirty() {
            " *"
        } else {
            ""
        };

        if let Some(filename) = self.document.get_file_name() {
            file_name = filename;
            file_name.truncate(20);
        }

        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        let len = status.len() + line_indicator.len();

        status.push_str(&" ".repeat(width.saturating_sub(len)));

        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        self.terminal.set_fg_color(STATUS_FG_COLOR)?;
        self.terminal.set_bg_color(STATUS_BG_COLOR)?;
        println!("{}\r", status);

        Ok(())
    }

    fn draw_message_bar(&mut self) -> Result<(), std::io::Error> {
        self.terminal.clear_current_line()?;
        let message = &self.status_message;

        self.terminal.set_fg_color(STATUS_FG_COLOR)?;
        self.terminal.set_bg_color(STATUS_BG_COLOR)?;

        let width = self.terminal.get_size().width as usize;
        let mut text: String;

        if Instant::now() - message.time < Duration::new(5, 0) {
            let len = message.text.len();
            text = format!("{}{}", message.text.clone(), &" ".repeat(width - len));

        } else {
            text = " ".repeat(width);
        }

        text.truncate(width);
        print!("{}", text);

        self.terminal.reset_colors()?;

        if let Err(_) = self.terminal.flush() {
            panic!("Error flushing terminal.");
        }

        Ok(())
    }

    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, KeyCode, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            let key_event: KeyEvent = Terminal::read_key(|width, height| {
                self.terminal.set_size(width, height);
                self.scroll();
                self.refresh_screen()?;
                Ok(())
            })?;

            match key_event.code {
                KeyCode::Char(ch) => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        result.clear();
                        break;
                    } else {
                        result.push(ch);
                    }
                }
                KeyCode::Enter => break,
                KeyCode::Backspace => {
                    result.pop();
                }
                KeyCode::Esc => {
                    result.clear();
                    break;
                }
                _ => (),
            }
            callback(self, key_event.code, &result);
        }

        if self.should_quit == true {
            return Ok(None);
        }

        self.status_message = StatusMessage::from(String::new());

        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
}
