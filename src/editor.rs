use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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
        println!("Raw mode enabled! Press 'CTRL + Q' to quit.\r");

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
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

    pub fn draw_rows(&self) -> io::Result<()> {
        let (_terminal_width, terminal_height) = size()?;

        for row in 0..terminal_height {
            execute!(io::stdout(), MoveTo(0, row))?;
            print!("~");
        }

        execute!(io::stdout(), MoveTo(0, 0))?;
        io::stdout().flush()?;

        Ok(())
    }
}