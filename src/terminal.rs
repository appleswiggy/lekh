use std::{
    io::{stdout, Stdout, Write},
    process,
};

use crossterm::{
    event::{read, Event, KeyEvent},
    style::Color,
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};

use crate::Position;

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: Stdout,
}

impl Terminal {
    pub fn default() -> Self {
        let (columns, rows) = if let Ok(size) = crossterm::terminal::size() {
            size
        } else {
            eprintln!("Error: Couldn't fetch terminal state.\r");
            process::exit(101);
        };

        let size: Size = Size {
            width: columns,
            height: rows.saturating_sub(2),
        };

        Terminal {
            size,
            _stdout: stdout(),
        }
    }

    pub fn get_size(&self) -> &Size {
        &self.size
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        self.size = Size {
            width,
            height: height.saturating_sub(2),
        };
    }

    pub fn into_raw_mode() {
        match is_raw_mode_enabled() {
            Ok(enabled) => {
                if !enabled && enable_raw_mode().is_err() {
                    Terminal::cleanup_and_exit(Some("Error: Couldn't enable raw mode."), 101);
                }
            }
            Err(_) => Terminal::cleanup_and_exit(Some("Error: Couldn't fetch terminal state."), 101),
        }
    }

    pub fn clear_screen(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(
            self._stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        )?;
        Ok(())
    }

    pub fn clear_current_line(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(
            self._stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
        )?;
        Ok(())
    }

    pub fn move_cursor(&mut self, position: &Position) -> Result<(), std::io::Error> {
        let Position { x, y } = position;
        let x = *x as u16;
        let y = *y as u16;
        crossterm::execute!(self._stdout, crossterm::cursor::MoveTo(x, y),)?;

        Ok(())
    }

    pub fn cursor_hide(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(self._stdout, crossterm::cursor::Hide)?;
        Ok(())
    }

    pub fn cursor_show(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(self._stdout, crossterm::cursor::Show)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self._stdout.flush()?;
        Ok(())
    }

    pub fn read_key<C>(mut resize_handler: C) -> Result<KeyEvent, std::io::Error>
    where
        C: FnMut(u16, u16) -> Result<(), std::io::Error>,
    {
        loop {
            let event = read()?;
            match event {
                Event::Key(key_event) => return Ok(key_event),
                Event::Resize(width, height) => {
                    resize_handler(width, height)?;
                }
                _ => (),
            }
        }
    }

    pub fn set_bg_color(&mut self, color: Color) -> Result<(), std::io::Error> {
        crossterm::execute!(self._stdout, crossterm::style::SetBackgroundColor(color))?;
        Ok(())
    }

    pub fn set_fg_color(&mut self, color: Color) -> Result<(), std::io::Error> {
        crossterm::execute!(self._stdout, crossterm::style::SetForegroundColor(color))?;
        Ok(())
    }

    pub fn reset_colors(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(self._stdout, crossterm::style::ResetColor)?;
        Ok(())
    }

    pub fn enter_alternate_screen() -> Result<(), std::io::Error> {
        crossterm::execute!(stdout(), crossterm::terminal::EnterAlternateScreen)?;
        Ok(())
    }

    pub fn leave_alternate_screen() -> Result<(), std::io::Error> {
        crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn cleanup_and_exit(err: Option<&str>, mut exit_code: i32) -> ! {
        if Terminal::leave_alternate_screen().is_err() {
            eprintln!("Error: Couldn't leave alternate screen.\r");
            if exit_code == 0 {
                exit_code = 102;
            }
        }

        if let Some(message) = err {
            eprintln!("{}\r", message);
        }

        match is_raw_mode_enabled() {
            Ok(enabled) => {
                if enabled && disable_raw_mode().is_err() {
                    eprintln!("Error: Couldn't disable raw mode.\r");

                    if exit_code == 0 {
                        exit_code = 101;
                    }
                }
            }
            Err(_) => {
                eprintln!("Error: Couldn't fetch terminal state.\r");

                if exit_code == 0 {
                    exit_code = 101;
                }
            },
        }
        
        process::exit(exit_code);
    }
}
