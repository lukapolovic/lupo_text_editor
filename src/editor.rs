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

pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
        }
    }

    pub fn load(&mut self, filename: &str) -> io::Result<()> {
        let file_contents = std::fs::read_to_string(filename)?;

        self.lines = file_contents.lines().map(|s| s.to_string()).collect();

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

pub struct View {
    pub buffer: Buffer,
}

impl View {
    pub fn render(&self, terminal_size: Size) -> io::Result<()> {
        queue!(io::stdout(), Hide)?;

        for row in 0..terminal_size.height {
            queue!(
                io::stdout(),
                MoveTo(0, row),
                Clear(ClearType::CurrentLine),
                Print("~")
            )?;
        }

        if self.buffer.is_empty() {
            self.draw_welcome_msg(terminal_size)?;
        } else {
            for (index, line) in self.buffer.lines.iter().enumerate() {
                let y = index as u16;
                if y < terminal_size.height {
                    if let Some(visible_line) = line.get(0..terminal_size.width as usize) {
                        queue!(
                            io::stdout(),
                            MoveTo(0, y),
                            Clear(ClearType::CurrentLine),
                            Print(visible_line)
                        )?;
                    }
                }
            }
        }

        queue!(io::stdout(), MoveTo(0, 0), Show)?;
        io::stdout().flush()?;
        Ok(())
    }

    pub fn draw_welcome_msg(&self, terminal_size: Size) -> io::Result<()> {
        let message = "Lupo - Lightweight Rust Text Editor (v1.0)";
        let message_length = message.len() as u16;

        let y_pos = terminal_size.height - 1;
        let x_pos = (terminal_size.width / 2) - (message_length / 2);

        let welcome_msg_coords = Size { width: x_pos, height: y_pos };

        queue!(io::stdout(), MoveTo(welcome_msg_coords.width, welcome_msg_coords.height), Print(message), MoveTo(0, 0))?;
        io::stdout().flush()?;

        Ok(())
    }
}

pub struct Editor {
    pub should_quit: bool,
    pub cursor_position: Position,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            should_quit: false, 
            cursor_position: { Position {x: 0, y: 0} },
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;

        let args: Vec<String> = std::env::args().collect();
        let mut view = View { buffer: Buffer::new() };

        if let Some(filename) = args.get(1) {
            if let Err(e) = view.buffer.load(filename) {
                eprintln!("Error loading file: {}", e);
                return Err(e);
            }
        }

        let (width, height) = size()?;
        let terminal_size = Size { width, height };
        view.render(terminal_size);

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
                        self.cursor_position.y += 1;
                        self.cursor_position.x = 0;
                    }
                    KeyCode::Up => {
                        if self.cursor_position.y > 0 {
                            self.cursor_position.y -= 1;
                        }
                        queue!(io::stdout(), MoveTo(self.cursor_position.x, self.cursor_position.y))?;
                        io::stdout().flush()?;
                    }
                    KeyCode::Down => {
                        let (_width, height) = size()?;
                        if self.cursor_position.y < height - 1 {
                            self.cursor_position.y += 1;
                        }
                        queue!(io::stdout(), MoveTo(self.cursor_position.x, self.cursor_position.y))?;
                        io::stdout().flush()?;
                    }
                    KeyCode::Left => {
                        if self.cursor_position.x > 0 {
                            self.cursor_position.x -= 1;
                        }
                        queue!(io::stdout(), MoveTo(self.cursor_position.x, self.cursor_position.y))?;
                        io::stdout().flush()?;
                    }
                    KeyCode::Right => {
                        let (width, _height) = size()?;
                        if self.cursor_position.x < width - 1 {
                            self.cursor_position.x += 1;
                        }
                        queue!(io::stdout(), MoveTo(self.cursor_position.x, self.cursor_position.y))?;
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