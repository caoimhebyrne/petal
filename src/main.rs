use std::{env, fs, process};

use colored::Colorize;

use crate::{ast::ASTParser, core::error::Error, lexer::Lexer};

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
                print_error(file_name, contents, error);
                process::exit(1);
            }
        };

        println!("{:?}", node);
    }
}

fn print_error(file_name: &str, contents: &str, error: Error) {
    let (error_line_number, error_column_number) = error.span.get_line_and_column(contents);

    eprintln!(
        "{}: {}",
        format!(
            "{}({}:{}:{})",
            String::from("error").red().bold(),
            file_name,
            error_line_number,
            error_column_number
        )
        .white(),
        format!("{}", error).bold(),
    );

    // In order to print some more context, we can attempt to print the line before the one that the error occurred on.
    if let Some(line) = contents.lines().nth(error_line_number - 2) {
        eprintln!("{} {}", format!("{} |", error_line_number - 1).white(), line.white());
    }

    // We can then print the line that the error was found on, with some carets underneath to indicate the token that
    // caused the error.
    if let Some(line) = contents.lines().nth(error_line_number - 1) {
        eprintln!("{} {}", format!("{} |", error_line_number).white(), line.bright_white());

        eprintln!(
            "    {}{}",
            " ".repeat(error_column_number - 1),
            "^".repeat(error.span.length()).red().bold()
        );
    }
}
