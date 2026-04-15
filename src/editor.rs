use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

static RAW_MODE_ENABLED: AtomicBool = AtomicBool::new(false);
static ALTERNATE_SCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);
pub(crate) static TERMINAL_CLEANED_UP: AtomicBool = AtomicBool::new(false);

struct TerminalGuard {
    raw_mode_enabled: bool,
    alternate_screen_active: bool,
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        // Start with nothing enabled
        let mut guard = Self {
            raw_mode_enabled: false,
            alternate_screen_active: false,
        };

        // Enable raw mode first
        enable_raw_mode()?;
        guard.raw_mode_enabled = true;
        RAW_MODE_ENABLED.store(true, Ordering::SeqCst);

        // Enter alternate screen
        queue!(io::stdout(), EnterAlternateScreen)?;
        guard.alternate_screen_active = true;
        ALTERNATE_SCREEN_ACTIVE.store(true, Ordering::SeqCst);

        TERMINAL_CLEANED_UP.store(false, Ordering::SeqCst);
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if !TERMINAL_CLEANED_UP.swap(true, Ordering::SeqCst) {
            // Clean up in reverse order
            if self.alternate_screen_active {
                let _ = queue!(io::stdout(), LeaveAlternateScreen, Show);
                ALTERNATE_SCREEN_ACTIVE.store(false, Ordering::SeqCst);
            }
            if self.raw_mode_enabled {
                let _ = disable_raw_mode();
                RAW_MODE_ENABLED.store(false, Ordering::SeqCst);
            }
            let _ = io::stdout().flush();
        }
    }
}

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
        self.lines = file_contents.lines().map(|s| s.to_string()).collect();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn get_byte_index(&self, line_idx: usize, char_idx: usize) -> usize {
        let line = &self.lines[line_idx];
        line.char_indices()
            .nth(char_idx)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(line.len())
    }

    pub fn get_char_count(&self, line_idx: usize) -> u16 {
        let line = &self.lines[line_idx];
        line.chars().count() as u16
    }
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
        if self.needs_redraw {
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
                        let char_limit = std::cmp::min(
                            self.buffer.get_char_count(index) as usize,
                            terminal_size.width as usize,
                        );
                        let byte_end = self.buffer.get_byte_index(index, char_limit);
                        let slice = &line[0..byte_end];
                        queue!(io::stdout(), MoveTo(0, y), Print(slice))?;
                    }
                }
            }
            self.needs_redraw = false;
        }

        queue!(
            io::stdout(),
            MoveTo(cursor_position.x, cursor_position.y),
            Show
        )?;
        io::stdout().flush()?;
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
        let _guard = TerminalGuard::new()?;
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
                    let line_len = view.buffer.get_char_count(self.cursor_position.y as usize);
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
                        let byte_idx = view.buffer.get_byte_index(
                            self.cursor_position.y as usize,
                            self.cursor_position.x as usize,
                        );
                        let line = &mut view.buffer.lines[self.cursor_position.y as usize];
                        line.insert(byte_idx, c);
                        self.cursor_position.x += 1;
                        view.needs_redraw = true;
                    }
                    KeyCode::Enter => {
                        let current_line =
                            view.buffer.lines[self.cursor_position.y as usize].clone();
                        let split_at = self.cursor_position.x as usize;
                        let remaining_text: Vec<char> =
                            current_line.chars().skip(split_at).collect();
                        view.buffer.lines[self.cursor_position.y as usize].truncate(split_at);
                        view.buffer
                            .lines
                            .push(remaining_text.iter().cloned().collect::<String>());
                        self.cursor_position.y += 1;
                        self.cursor_position.x = 0;
                        view.needs_redraw = true;
                    }
                    KeyCode::Up => {
                        if self.cursor_position.y > 0 {
                            self.cursor_position.y -= 1;
                            let line_len =
                                view.buffer.lines[self.cursor_position.y as usize].len() as u16;
                            if self.cursor_position.x > line_len {
                                self.cursor_position.x = line_len;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if self.cursor_position.y
                            < (view.buffer.lines.len() as u16).saturating_sub(1)
                        {
                            self.cursor_position.y += 1;
                            let line_len =
                                view.buffer.lines[self.cursor_position.y as usize].len() as u16;
                            if self.cursor_position.x > line_len {
                                self.cursor_position.x = line_len;
                            }
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor_position.x > 0 {
                            self.cursor_position.x -= 1;
                        }
                    }
                    KeyCode::Right => {
                        let line_len =
                            view.buffer.lines[self.cursor_position.y as usize].len() as u16;
                        if self.cursor_position.x < line_len {
                            self.cursor_position.x += 1;
                        }
                    }
                    _ => {}
                }
            }

            view.render(terminal_size, self.cursor_position)?;

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
}
