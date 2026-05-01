use std::{
    ffi::OsStr,
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{
    ast::statement::StatementKind,
    backend::c::CBackend,
    core::error::Error,
    module::ParsedModule,
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
    #[arg(required = true, num_args=1..)]
    input: Vec<String>,
}

fn create_and_parse_module(
    parsed_modules: &mut Vec<ParsedModule>,
    module_registry: &mut ModuleRegistry,
    file_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let module_id = module_registry.create_module(file_path.clone())?;
    let parsed_module = module_registry.get_module(module_id).parse()?;

    for statement in &parsed_module.ast {
        // If this is an import statement, then we must be able to find a module with the imported name in the
        // same directory.
        if let StatementKind::Import(import) = &statement.kind {
            let imported_module_path = file_path.with_file_name(import.name.clone()).with_extension("petal");
            create_and_parse_module(parsed_modules, module_registry, imported_module_path)?;
        }
    }

    parsed_modules.push(parsed_module);
    Ok(())
}

fn main_impl(args: Args, module_registry: &mut ModuleRegistry) -> Result<(), Box<dyn Error>> {
    println!("{} Parsing modules", "[1/4]".bright_purple());

    let mut parsed_modules: Vec<ParsedModule> = Vec::new();

    for file_path in &args.input {
        create_and_parse_module(&mut parsed_modules, module_registry, PathBuf::from(file_path))?;
    }

    println!("{} Checking types", "[2/4]".bright_purple());

    let mut typechecker = Typechecker::default();
    let checked_modules = typechecker.check(parsed_modules)?;

    println!("{} Generating C code", "[3/4]".bright_purple());

    let code = CBackend::emit_code(&checked_modules)?;
    if args.emit_code {
        println!("{code}");
    }

    // `./path/to/petal/file.petal` -> `file`
    let binary_file_name = args.output.unwrap_or_else(|| {
        // FIXME: There might be a better way to determine this instead of `args.input[0]`?
        PathBuf::from(args.input[0].clone()).file_stem().and_then(OsStr::to_str).unwrap_or("output").to_string()
    });

    if !args.no_emit_binary {
        println!("{} Compiling binary ('{binary_file_name}')", "[4/4]".bright_purple());
        CBackend::emit_binary(&code, binary_file_name)?;
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
