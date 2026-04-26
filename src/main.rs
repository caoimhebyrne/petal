use std::{
    env,
    ffi::OsStr,
    path::PathBuf,
};

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

fn main() {
    let mut args = env::args();

    let program_name = args.next().unwrap_or("petal".into());

    let file_path = match args.next() {
        Some(value) => value,
        _ => {
            eprintln!("Usage: {} file_path", program_name);
            return;
        }
    };

    println!("[1/4] Creating module from '{file_path}'");

    let module = match Module::create(file_path.clone()) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error);
            return;
        }
    };

    println!("[2/4] Parsing module");

    let parsed_module = match module.parse() {
        Ok(value) => value,
        Err(error) => {
            error.print_to_stderr(&module.file_path, &module.file_contents);
            return;
        }
    };

    println!("[3/4] Generating C code");

    let code = match CBackend::emit_code(&parsed_module) {
        Ok(value) => value,
        Err(error) => {
            error.print_to_stderr(&module.file_path, &module.file_contents);
            return;
        }
    };

    // `./path/to/petal/file.petal` -> `file`
    let binary_file_name = PathBuf::from(file_path).file_stem().and_then(OsStr::to_str).unwrap_or("output").to_string();

    println!("[4/4] Compiling binary ('{binary_file_name}')");

    if let Err(error) = CBackend::emit_binary(&code, binary_file_name) {
        error.print_to_stderr(&module.file_path, &module.file_contents);
    }
}
