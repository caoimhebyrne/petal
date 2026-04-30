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
    module_registry::ModuleRegistry,
    typechecker::Typechecker,
};

pub mod ast;
pub mod backend;
pub mod core;
pub mod lexer;
pub mod module;
pub mod module_registry;
pub mod typechecker;

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

fn main_impl(args: Args, module_registry: &mut ModuleRegistry) -> Result<(), Box<dyn Error>> {
    println!("{} Creating module from '{}'", "[1/5]".bright_purple(), args.input);

    let module_id = module_registry.create_module(args.input.clone())?;
    let module = module_registry.get_module(module_id);

    println!("{} Parsing module", "[2/5]".bright_purple());
    let parsed_module = module.parse()?;

    println!("{} Checking types", "[3/5]".bright_purple());

    let mut typechecker = Typechecker::default();
    let checked_module = typechecker.check(parsed_module)?;

    println!("{} Generating C code", "[4/5]".bright_purple());

    let code = CBackend::emit_code(&checked_module)?;
    if args.emit_code {
        println!("{code}");
    }

    // `./path/to/petal/file.petal` -> `file`
    let binary_file_name = args.output.unwrap_or_else(|| {
        PathBuf::from(args.input).file_stem().and_then(OsStr::to_str).unwrap_or("output").to_string()
    });

    if !args.no_emit_binary {
        println!("{} Compiling binary ('{binary_file_name}')", "[5/5]".bright_purple());

        CBackend::emit_binary(module.id, &code, binary_file_name)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();
    let mut module_registry = ModuleRegistry::default();

    if let Err(error) = main_impl(args, &mut module_registry) {
        error.print_to_stderr(&module_registry);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
