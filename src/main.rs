mod editor;

use crate::editor::TERMINAL_CLEANED_UP;
use crossterm::{cursor, queue, terminal};
use editor::Editor;
use std::io::Write;
use std::panic;
use std::sync::atomic::Ordering;

fn set_panic_hook() {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if !TERMINAL_CLEANED_UP.swap(true, Ordering::SeqCst) {
            // Best effort terminal cleanup
            let _ = terminal::disable_raw_mode();
            let _ = queue!(
                std::io::stdout(),
                terminal::LeaveAlternateScreen,
                cursor::Show
            );
            let _ = std::io::stdout().flush();
        }
        prev(panic_info);
    }));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_panic_hook();

    let mut editor = Editor::new();

    editor.run()?;

    println!("Goodbye!");
    Ok(())
}
