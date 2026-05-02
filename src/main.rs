use std::{
    ffi::OsStr,
    io::Write,
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;
use log::Level;
use pretty_env_logger::env_logger::fmt::Color;

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

#[macro_use]
extern crate log;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Whether the generated code should be written to stdout.
    #[arg(long)]
    emit_code: bool,

    /// Whether verbose output is enabled.
    #[arg(long, short)]
    verbose: bool,

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

            debug!(
                "Module {} imports '{}', which resolves to path '{}'",
                module_id,
                import.name,
                imported_module_path.to_string_lossy()
            );

            create_and_parse_module(parsed_modules, module_registry, imported_module_path)?;
        }
    }

    parsed_modules.push(parsed_module);
    Ok(())
}

fn main_impl(args: Args, module_registry: &mut ModuleRegistry) -> Result<(), Box<dyn Error>> {
    info!("Parsing modules");

    let mut parsed_modules: Vec<ParsedModule> = Vec::new();

    for file_path in &args.input {
        create_and_parse_module(&mut parsed_modules, module_registry, PathBuf::from(file_path))?;
    }

    info!("Checking types");

    let mut typechecker = Typechecker::default();
    let checked_modules = typechecker.check(parsed_modules)?;

    info!("Generating code");

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
        info!("Compiling binary ('{binary_file_name}')");
        CBackend::emit_binary(&code, binary_file_name)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    let default_log_level = if args.verbose { log::LevelFilter::Trace } else { log::LevelFilter::Info };

    pretty_env_logger::formatted_builder()
        .filter_level(default_log_level)
        .parse_default_env()
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_color(match record.level() {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Blue,
                Level::Trace => Color::Magenta,
            });

            let level = style.value(format!("{:>5}", record.level()));
            writeln!(buf, "{level}  {}", record.args())
        })
        .init();

    let mut module_registry = ModuleRegistry::default();

    if let Err(error) = main_impl(args, &mut module_registry) {
        error.print_to_stderr(&module_registry);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
