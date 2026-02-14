use std::{env, fs};

use crate::{core::error::Error, lexer::Lexer};

pub mod core;
pub mod lexer;

fn main() {
    let program_name = env::args().next().unwrap_or("petal".into());

    let file_path = match env::args().next() {
        Some(value) => value,
        _ => {
            eprintln!("Usage: {} file_path", program_name);
            return;
        }
    };

    let file_contents = match fs::read_to_string(&file_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: failed to read from '{}': {}", file_path, error);
            return;
        }
    };

    let mut lexer = Lexer::new(&file_contents);
    let tokens = match lexer.parse() {
        Ok(value) => value,
        Err(error) => {
            error.print_to_stderr(&file_path, &file_contents);

            return;
        }
    };

    println!("info: parsed source code from '{}', read {} token(s)", &file_path, tokens.len());
}
