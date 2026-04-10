use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};

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

        for b in io::stdin().bytes() {
            let c = b? as char;
            
            print!("{}", c);
            io::stdout().flush()?;

            if c == 'q' {
                self.should_quit = true;
                break;
            }
        }

        disable_raw_mode()?;
        Ok(())
    }
}