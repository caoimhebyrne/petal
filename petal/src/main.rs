#![feature(impl_trait_in_bindings, path_file_prefix, path_add_extension)]
#![allow(clippy::new_without_default)]

use clap::Parser;
use colored::Colorize;
use petal_codegen::{Aarch64MacOSDriver, Driver, X86_64LinuxDriver};
use petal_core::{ast::Ast, core::location::Location, lexer::Lexer, typechecker::Typechecker};
use petal_ir::generator::IRGenerator;
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Target {
    #[clap(name = "x86_64-linux")]
    #[default]
    X86_64Linux,

    #[clap(name = "aarch64-macos")]
    Aarch64MacOS,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long("output"))]
    output_path: PathBuf,

    #[arg(short, long("target"), default_value("x86_64-linux"))]
    target: Target,

    input_path: PathBuf,
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

    let mut ir_generator = IRGenerator::new();
    let functions = match ir_generator.generate(nodes) {
        Ok(value) => value,
        Err(error) => report_error(&args.input_path, &error, Some(error.location)),
    };

    println!("[5/5] Codegen");

    let mut driver: Box<dyn Driver> = match args.target {
        Target::X86_64Linux => Box::new(X86_64LinuxDriver::new()),
        Target::Aarch64MacOS => Box::new(Aarch64MacOSDriver::new()),
    };

    if let Err(error) = driver.generate(functions, &args.output_path) {
        report_error(&args.input_path, &error, error.location)
    };
}
