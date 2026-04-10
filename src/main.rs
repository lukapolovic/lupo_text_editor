use std::io::{self, Read, Write};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

fn main() {
    enable_raw_mode().unwrap();

    println!("Raw mode enabled! Press 'q' to quit.");

    for b in io::stdin().bytes() {
        let c = b.unwrap() as char;
        
        print!("{}", c);

        io::stdout().flush().unwrap();

        if c == 'q' {
            disable_raw_mode().unwrap();
            println!("\nQuitting...");
            break;
        }
    }
}