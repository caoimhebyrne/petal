use std::env;

use crate::module::Module;

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

    let module = match Module::create(file_path.clone()) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error);
            return;
        }
    };

    if let Err(error) = module.parse() {
        error.print_to_stderr(&module.file_path, &module.file_contents);
    }
}
