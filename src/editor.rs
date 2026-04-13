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

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: Vec::<String>::new(),
        }
    }

    pub fn load(&mut self, filename: &str) -> io::Result<()> {
        let file_contents = std::fs::read_to_string(filename)?;
        self.lines = file_contents
            .lines()
            .map(|s| s.to_string())
            .collect();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    //TODO get_byte_index
    /*
    Purpose: Translates a character position (e.g., "the 3rd character") into the exact byte where that character starts. This is
    what we'll use when we actually want to insert or truncate.

    Logic:
    1. Take line_idx: usize and char_idx: usize.
    2. Get the line: let line = &self.lines[line_idx];
    3. Use the iterator .char_indices() which returns (byte_index, character).
    4. Use .nth(char_idx) to skip to your target character.
    5. If it exists, return the byte_index.
    6. If it doesn't (you're at the end of the line), return the full line.len().*/

    //TODO get_char_count
    /*
    Purpose: Tells us how many characters are in a line. We need this so the Right arrow key knows when to stop. (The current
    line.len() tells us bytes, which is why it's currently buggy!)

    Logic:
    1. Take line_idx: usize.
    2. Get the line: let line = &self.lines[line_idx];
    3. Return the count of characters: line.chars().count() as u16.*/
}

#[derive(Default)]
pub struct View {
    pub buffer: Buffer,
    pub needs_redraw: bool,
}

impl View {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            needs_redraw: true,
        }
    }

    pub fn render(&mut self, terminal_size: Size, cursor_position: Position) -> io::Result<()> {
        queue!(io::stdout(), Clear(ClearType::All), Hide)?;

        for row in 0..terminal_size.height {
            queue!(io::stdout(), MoveTo(0, row), Print("~"))?;
        }

        if self.buffer.is_empty() {
            self.draw_welcome_msg(terminal_size)?;
        } else {
            for (index, line) in self.buffer.lines.iter().enumerate() {
                let y = index as u16;
                if y < terminal_size.height {
                    let end = std::cmp::min(line.len(), terminal_size.width as usize);
                    let slice = &line[0..end];
                    queue!(
                        io::stdout(),
                        MoveTo(0, y),
                        Print(slice)
                    )?;
                }
            }
        }

        queue!(io::stdout(), MoveTo(cursor_position.x, cursor_position.y), Show)?;
        io::stdout().flush()?;
        self.needs_redraw = false;
        Ok(())
    }

    pub fn draw_welcome_msg(&self, terminal_size: Size) -> io::Result<()> {
        let message = "Lupo - Lightweight Rust Text Editor (v1.0)";
        let message_len = message.chars().count() as u16;
        let y_pos = terminal_size.height - 1;
        let x_pos = (terminal_size.width / 2).saturating_sub(message_len / 2);
        queue!(io::stdout(), MoveTo(x_pos, y_pos), Print(message))?;
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
            cursor_position: Position { x: 0, y: 0 },
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let args: Vec<String> = std::env::args().collect();
        let mut view = View::new();

        if let Some(filename) = args.get(1) {
            if let Err(e) = view.buffer.load(filename) {
                eprintln!("Error loading file: {}", e);
                return Err(e);
            }
        }

        let (width, height) = size()?;
        let terminal_size = Size { width, height };
        view.render(terminal_size, self.cursor_position)?;

        loop {
            if let Event::Key(key_event) = event::read()? {
                if !view.buffer.lines.is_empty() {
                    let line_len = view.buffer.lines[self.cursor_position.y as usize].len() as u16;
                    if self.cursor_position.x > line_len {
                        self.cursor_position.x = line_len;
                    }
                }

                match key_event.code {
                    KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                        break;
                    }
                    KeyCode::Char(c) => {
                        if view.buffer.lines.is_empty() {
                            view.buffer.lines.push(String::new());
                        }
                        let line = &mut view.buffer.lines[self.cursor_position.y as usize];
                        line.insert(self.cursor_position.x as usize, c);
                        self.cursor_position.x += 1;
                        view.needs_redraw = true;
                    }
                    KeyCode::Enter => {
                        let current_line = view.buffer.lines[self.cursor_position.y as usize].clone();
                        let split_at = self.cursor_position.x as usize;
                        let remaining_text: Vec<char> = current_line.chars().skip(split_at).collect();
                        view.buffer.lines[self.cursor_position.y as usize].truncate(split_at);
                        view.buffer.lines.push(remaining_text.iter().cloned().collect::<String>());
                        self.cursor_position.y += 1;
                        self.cursor_position.x = 0;
                        view.needs_redraw = true;
                    }
                    KeyCode::Up => {
                        if self.cursor_position.y > 0 {
                            self.cursor_position.y -= 1;
                            let line_len = view.buffer.lines[self.cursor_position.y as usize].len() as u16;
                            if self.cursor_position.x > line_len {
                                self.cursor_position.x = line_len;
                            }
                        }
                        view.needs_redraw = true;
                    }
                    KeyCode::Down => {
                        if self.cursor_position.y < (view.buffer.lines.len() as u16).saturating_sub(1) {
                            self.cursor_position.y += 1;
                            let line_len = view.buffer.lines[self.cursor_position.y as usize].len() as u16;
                            if self.cursor_position.x > line_len {
                                self.cursor_position.x = line_len;
                            }
                        }
                        view.needs_redraw = true;
                    }
                    KeyCode::Left => {
                        if self.cursor_position.x > 0 {
                            self.cursor_position.x -= 1;
                        }
                        view.needs_redraw = true;
                    }
                    KeyCode::Right => {
                        let line_len = view.buffer.lines[self.cursor_position.y as usize].len() as u16;
                        if self.cursor_position.x < line_len {
                            self.cursor_position.x += 1;
                        }
                        view.needs_redraw = true;
                    }
                    _ => {}
                }
            }

            if view.needs_redraw {
                view.render(terminal_size, self.cursor_position)?;
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        Ok(())
    }
}