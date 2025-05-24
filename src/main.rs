#![feature(path_file_prefix, path_add_extension)]
#![allow(clippy::new_without_default)]

use core::location::Location;
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use ast::Ast;
use clap::Parser;
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

#[derive(Parser)]
struct Args {
    #[arg(short, long("output"))]
    output_path: PathBuf,

    path: PathBuf,
}

fn report_error(path: &Path, error: impl Display, location: Option<Location>) -> ! {
    let location_string = match location {
        Some(location) => format!("{}:{}", location.line + 1, location.column + 1),
        None => "unknown".to_owned(),
    };

    println!(
        "{}{} {}",
        "error".red(),
        format!("({}:{}):", path.to_string_lossy(), location_string).white(),
        error
    );

    exit(-1);
}

fn main() {
    let args = Args::parse();
    let file_contents = match fs::read_to_string(args.path.clone()) {
        Ok(value) => value,
        Err(error) => return eprintln!("ERROR: {}", error),
    };

    let mut lexer = Lexer::new(&file_contents);
    let tokens = match lexer.parse() {
        Ok(value) => value,
        Err(error) => report_error(&args.path, &error, Some(error.location)),
    };

    let mut ast = Ast::new(tokens);
    let mut nodes = match ast.parse() {
        Ok(value) => value,
        Err(error) => report_error(&args.path, &error, error.location),
    };

    let mut typechecker = Typechecker::new(&mut nodes);
    if let Err(error) = typechecker.check() {
        report_error(&args.path, &error, error.location)
    };

    let codegen_context = Context::create();
    let mut codegen = Codegen::new(&args.output_path, &codegen_context, &nodes);
    if let Err(error) = codegen.compile() {
        report_error(&args.path, &error, error.location)
    };
}
