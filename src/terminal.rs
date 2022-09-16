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
    pub fn default() -> Result<Self, std::io::Error> {
        let (columns, rows) = crossterm::terminal::size()?;
        let size: Size = Size {
            width: columns,
            height: rows.saturating_sub(2),
        };

        Ok(Terminal {
            size,
            _stdout: stdout(),
        })
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
                    // the same below note applies here as well.
                    // write eprintln instead of panic and use process::exit
                    panic!("Error enabling raw mode.");
                }
            }
            // note - raw mode is not enabled yet, so don't try to disable it here.
            // or add a check in cleanup function - if raw mode is enabled - then disable it
            Err(_) => panic!("Error fetching terminal state."),
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

    pub fn enter_alternate_screen(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(self._stdout, crossterm::terminal::EnterAlternateScreen)?;
        Ok(())
    }

    pub fn leave_alternate_screen(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(self._stdout, crossterm::terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn cleanup_and_exit(&mut self, status_code: i32) {
        match is_raw_mode_enabled() {
            Ok(enabled) => {
                if enabled && disable_raw_mode().is_err() {
                    panic!("Error disabling raw mode.");
                }
            }
            Err(_) => panic!("Error fetching terminal state."),
        }
        if let Err(_) = self.leave_alternate_screen() {
            panic!("Error leaving alternate screen.");
        }
        process::exit(status_code);
    }
}
