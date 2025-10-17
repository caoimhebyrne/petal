use std::{env, fs, process};

use crate::{ast::ASTParser, lexer::Lexer};

pub mod ast;
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

    dump_ast(&file_path, &contents);
}

fn dump_ast<'a>(file_name: &'a str, contents: &'a str) {
    let mut lexer = Lexer::new(&contents);
    let mut ast_parser = ASTParser::new(&mut lexer);

    loop {
        let node = match ast_parser.next_statement() {
            Ok(value) => value,
            Err(error) => {
                let (line, column) = error.span.get_line_and_column(contents);
                eprintln!("error({}:{}:{}): {}", file_name, line, column, error);

                process::exit(1);
            }
        };

        println!("{:?}", node);
    }
}
