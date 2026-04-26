use std::{
    env,
    ffi::OsStr,
    path::PathBuf,
    process::ExitCode,
};

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

fn main() -> ExitCode {
    let mut args = env::args();

    let program_name = args.next().unwrap_or("petal".into());

    let file_path = match args.next() {
        Some(value) => value,
        _ => {
            eprintln!("Usage: {} file_path", program_name);
            return ExitCode::FAILURE;
        }
    };

    println!("{} Creating module from '{file_path}'", "[1/4]".bright_purple());

    let module = match Module::create(file_path.clone()) {
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

    // `./path/to/petal/file.petal` -> `file`
    let binary_file_name = PathBuf::from(file_path).file_stem().and_then(OsStr::to_str).unwrap_or("output").to_string();

    println!("{} Compiling binary ('{binary_file_name}')", "[4/4]".bright_purple());

    if let Err(error) = CBackend::emit_binary(&code, binary_file_name) {
        error.print_to_stderr(&module.file_path, &module.file_contents);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
