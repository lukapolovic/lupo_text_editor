use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::event::{self, Event, KeyCode};
use std::io::{self, Write};

pub struct Editor {
    pub should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {should_quit: false}
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        println!("Raw mode enabled! Press 'q' to quit.\r");

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                        break;
                    }
                    KeyCode::Char(c) => {
                        print!("You pressed: {c} \r\n");
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
}