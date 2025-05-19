use core::location::Location;
use std::{fmt::Display, fs, process::exit};

use ast::AST;
use codegen::Codegen;
use colored::Colorize;
use inkwell::context::Context;
use lexer::Lexer;
use typechecker::Typechecker;

pub mod ast;
pub mod codegen;
pub mod core;
pub mod lexer;
pub mod typechecker;

fn report_error(error: impl Display, location: Option<Location>) -> ! {
    let location_string = match location {
        Some(location) => format!("{}:{}", location.line + 1, location.column + 1),
        None => "unknown".to_owned(),
    };

    println!(
        "{}{} {}",
        "error".red(),
        format!(
            "({}:{}):",
            "./examples/00_hello_world.petal", location_string
        )
        .white(),
        error
    );

    exit(-1);
}

fn main() {
    let file_contents = fs::read_to_string("./examples/00_hello_world.petal").unwrap();

    let mut lexer = Lexer::from(&file_contents);
    let tokens = match lexer.parse() {
        Ok(value) => value,
        Err(error) => report_error(&error, Some(error.location)),
    };

    let mut ast = AST::new(&tokens);
    let mut nodes = match ast.parse() {
        Ok(value) => value,
        Err(error) => report_error(&error, error.location),
    };

    let mut typechecker = Typechecker::new(&mut nodes);
    if let Err(error) = typechecker.check() {
        report_error(&error, error.location)
    };

    let codegen_context = Context::create();
    let codegen = Codegen::new("00_hello_world", &codegen_context, &nodes);
    codegen.compile();
}
