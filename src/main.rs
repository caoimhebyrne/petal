use std::{env, process};

use colored::Colorize;

use crate::{
    ast::{ASTParser, visitor::dump_visitor::DumpASTVisitor},
    core::{error::Error, module::Module},
    lexer::Lexer,
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

    let mut module = Module::new(&file_path).expect("failed to create module");
    dump_ast(&mut module);
}

fn dump_ast(module: &mut Module) {
    let mut lexer = Lexer::new(module.string_intern_pool.as_mut(), &module.contents);

    let token_stream = match lexer.get_stream() {
        Ok(value) => value,
        Err(error) => {
            print_error(&module, error);
            process::exit(1);
        }
    };

    let mut ast_parser = ASTParser::new(module.string_intern_pool.as_mut(), token_stream);
    let visitor = DumpASTVisitor::new();

    if let Err(error) = ast_parser.parse(&visitor) {
        print_error(&module, error);
        process::exit(1);
    }
}

fn print_error(module: &Module, error: Error) {
    let (error_line_number, error_column_number) = error.span.get_line_and_column(&module.contents);

    eprintln!(
        "{}: {}",
        format!(
            "{}({}:{}:{})",
            String::from("error").red().bold(),
            module.file_path,
            error_line_number,
            error_column_number
        )
        .white(),
        format!("{}", error).bold(),
    );

    // In order to print some more context, we can attempt to print the line before the one that the error occurred on.
    if error_line_number > 2
        && let Some(line) = module.contents.lines().nth(error_line_number - 2)
    {
        eprintln!("{} {}", format!("{} |", error_line_number - 1).white(), line.white());
    }

    // We can then print the line that the error was found on, with some carets underneath to indicate the token that
    // caused the error.
    if let Some(line) = module.contents.lines().nth(error_line_number - 1) {
        eprintln!("{} {}", format!("{} |", error_line_number).white(), line.bright_white());

        eprintln!(
            "    {}{}",
            " ".repeat(error_column_number - 1),
            "^".repeat(error.span.length()).red().bold()
        );
    }
}
