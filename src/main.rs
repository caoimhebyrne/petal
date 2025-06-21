#![feature(impl_trait_in_bindings, path_file_prefix, path_add_extension)]
#![allow(clippy::new_without_default)]

use core::location::Location;
use driver::Driver;
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use ast::Ast;
use clap::Parser;
use colored::Colorize;
use lexer::Lexer;
use typechecker::Typechecker;

use crate::{
    driver::{aarch64::Aarch64Driver, x86_64::X86_64Driver},
    ir::generator::IntermediateRepresentation,
};

pub mod ast;
pub mod core;
pub mod driver;
pub mod ir;
pub mod lexer;
pub mod typechecker;

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Target {
    Aarch64,
    #[default]
    X86_64,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long("output"))]
    output_path: PathBuf,

    #[arg(short, long("target"), default_value("x86-64"))]
    target: Target,

    input_path: PathBuf,
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Aarch64 => write!(f, "aarch64"),
            Target::X86_64 => write!(f, "x86_64"),
        }
    }
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
    let file_contents = match fs::read_to_string(args.input_path.clone()) {
        Ok(value) => value,
        Err(error) => return eprintln!("ERROR: {}", error),
    };

    println!("[1/5] Lexer");

    let mut lexer = Lexer::new(&file_contents);
    let tokens = match lexer.parse() {
        Ok(value) => value,
        Err(error) => report_error(&args.input_path, &error, Some(error.location)),
    };

    println!("[2/5] AST");

    let mut ast = Ast::new(tokens);
    let mut nodes = match ast.parse() {
        Ok(value) => value,
        Err(error) => report_error(&args.input_path, &error, error.location),
    };

    println!("[3/5] Typechecker");

    let mut typechecker = Typechecker::new(&mut nodes);
    if let Err(error) = typechecker.check() {
        report_error(&args.input_path, &error, Some(error.location))
    };

    println!("[4/5] IR Generator");

    let mut intermediate_representation = IntermediateRepresentation::new();
    let functions = match intermediate_representation.parse(&nodes) {
        Ok(value) => value,
        Err(error) => report_error(&args.input_path, &error, error.location),
    };

    println!("[5/5] Codegen for {}", args.target);

    let driver: Box<dyn Driver> = match args.target {
        Target::Aarch64 => Box::new(Aarch64Driver::new(args.output_path.clone())),
        Target::X86_64 => Box::new(X86_64Driver::new(args.output_path.clone())),
    };

    driver.compile(functions);

    println!("Compilation successful: {}", args.output_path.to_string_lossy());
}
