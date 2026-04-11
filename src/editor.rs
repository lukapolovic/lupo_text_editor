use crossterm::cursor::{Hide, Show, MoveTo};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io::{self, Write};

#[derive(Debug, Copy, Clone)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub struct Editor {
    pub should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {should_quit: false}
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        println!("Raw mode enabled! Press 'CTRL + Q' to quit.\r");

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                        break;
                    }
                    KeyCode::Char(c) => {
                        queue!(io::stdout(), Print(format!("You pressed: {c} \r\n")))?;
                        io::stdout().flush()?;
                    }
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        Ok(())
    }

    pub fn draw_rows(&self) -> io::Result<()> {
        let (width, height) = size()?;
        let terminal_size = Size { width: width, height: height };

        queue!(io::stdout(), Hide)?;

        for row in 0..terminal_size.height {
            queue!(io::stdout(), MoveTo(0, row), Clear(ClearType::CurrentLine), Print("~"))?;
        }

        queue!(io::stdout(), MoveTo(0, 0), Show)?;
        io::stdout().flush()?;

        Ok(())
    }
}