use std::{
    ffi::OsStr,
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{
    backend::c::CBackend,
    core::error::Error,
    module::Module,
};

pub mod ast;
pub mod backend;
pub mod core;
pub mod lexer;
pub mod module;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Whether the generated code should be written to stdout.
    #[arg(long)]
    emit_code: bool,

    /// Whether the compiler should skip emitting a binary.
    #[arg(long)]
    no_emit_binary: bool,

    /// The path to write the final executable to.
    #[arg(short, long)]
    output: Option<String>,

    /// The path to the source code to read from.
    #[arg()]
    input: String,
}

fn main() -> ExitCode {
    let args = Args::parse();

    println!("{} Creating module from '{}'", "[1/4]".bright_purple(), args.input);

    let module = match Module::create(args.input.clone()) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error);
            return ExitCode::FAILURE;
        }
    };

    println!("{} Parsing module", "[2/4]".bright_purple());

    let parsed_module = match module.parse() {
        Ok(value) => value,
        Err(error) => {
            error.print_to_stderr(&module.file_path, &module.file_contents);
            return ExitCode::FAILURE;
        }
    };

    println!("{} Generating C code", "[3/4]".bright_purple());

    let code = match CBackend::emit_code(&parsed_module) {
        Ok(value) => value,
        Err(error) => {
            error.print_to_stderr(&module.file_path, &module.file_contents);
            return ExitCode::FAILURE;
        }
    };

    if args.emit_code {
        println!("{code}");
    }

    // `./path/to/petal/file.petal` -> `file`
    let binary_file_name = args.output.unwrap_or_else(|| {
        PathBuf::from(args.input).file_stem().and_then(OsStr::to_str).unwrap_or("output").to_string()
    });

    if !args.no_emit_binary {
        println!("{} Compiling binary ('{binary_file_name}')", "[4/4]".bright_purple());

        if let Err(error) = CBackend::emit_binary(&code, binary_file_name) {
            error.print_to_stderr(&module.file_path, &module.file_contents);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
