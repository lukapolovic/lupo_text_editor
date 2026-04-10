use std::io::{self, Read, Write};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

fn main() {
    enable_raw_mode().unwrap();

    println!("Raw mode enabled! Press 'q' to quit.\r");

    for b_result in io::stdin().bytes() {
        match b_result {
            Ok(b) => {
                let c = b as char;

                if c.is_control() {
                    print!("Binary: {0:08b} ASCII: {0:#03}\r", b);
                } else {
                    print!("Binary: {0:08b} ASCII: {0:#03} Character: {1:#?}\r", b, c);
                }

                io::stdout().flush().unwrap();

                if c == 'q' {
                    break;
                }
            }

            Err(err) => {
                println!("\nAn error occurred: {}", err);
                break;
            }
        }
    }

    disable_raw_mode().unwrap();
}