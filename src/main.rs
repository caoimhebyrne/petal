use crate::{ast::ASTParser, core::error::Error, lexer::Lexer};
use std::{
    env,
    fmt::{Debug, Display},
    fs, process,
};

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
        let node = match ast_parser.next_node() {
            Ok(value) => value,
            Err(error) => {
                print_error(file_name, contents, error);
                process::exit(1);
            }
        };
        println!("{:?}", node);
    }
}

fn print_error<'a, K: Debug + Display>(file_name: &'a str, _contents: &'a str, error: Error<K>) {
    println!("error({}): {}", file_name, error.kind)
}
