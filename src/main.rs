use crate::lexer::Lexer;
use std::{env, fs, process};

pub mod core;
pub mod lexer;

/// This is the entrypoint for the Petal compiler.
fn main() {
    let mut args = env::args();

    // The first argument to the program must always be the name of the binary.
    let program_name = args.next().unwrap_or_else(|| "petal".into());

    // Then, we must have a file path to read from.
    let file_path = match args.next() {
        Some(value) => value,

        None => {
            eprintln!("Usage: {} <file_path>", program_name);
            process::exit(1);
        }
    };

    let contents = match fs::read_to_string(&file_path) {
        Ok(value) => value,

        Err(error) => {
            eprintln!("ERROR: Failed to read from file '{}': {}", file_path, error);
            process::exit(1);
        }
    };

    let mut _lexer = Lexer::new(&contents);

    // TODO: Pass the Lexer to the AST generator,
}
