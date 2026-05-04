use std::{
    env,
    fs,
    io::Write,
    path::PathBuf,
    process::{
        Command,
        ExitCode,
        exit,
    },
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
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

#[cfg(test)]
pub mod integration_tests;

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

    /// Whether run mode should be enabled.
    #[arg(long, short)]
    run: bool,

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

fn main_impl(mut args: Args, module_registry: &mut ModuleRegistry) -> Result<(), Box<dyn Error>> {
    if args.no_emit_binary && args.run {
        warn!("--no-emit-binary and --run provided as arguments, this is not supported. Ignoring --no-emit-binary!");
        args.no_emit_binary = false;
    }

    info!("Parsing modules");

    let mut parsed_modules: Vec<ParsedModule> = Vec::new();

    for file_path in &args.input {
        create_and_parse_module(&mut parsed_modules, module_registry, PathBuf::from(file_path))?;
    }

    info!("Checking types");

    let mut typechecker = Typechecker::default();
    let (checked_modules, declared_structures, declared_functions, optional_types) =
        typechecker.check(parsed_modules)?;

    info!("Generating code");

    let backend = CBackend::new(declared_structures, declared_functions, optional_types);
    let code = backend.emit_code(&checked_modules)?;
    if args.emit_code {
        println!("{code}");
    }

    // `./path/to/petal/file.petal` -> `file`
    let executable_file_path = if args.run {
        let current_timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("SystemTime::duration_since should not fail");

        let mut path = env::temp_dir();
        path.push(format!("petal-{}", current_timestamp.as_millis()));
        path
    } else {
        let path = args.output.unwrap_or_else(|| args.input[0].clone());
        PathBuf::from(path).with_extension("")
    };

    if !args.no_emit_binary {
        info!("Compiling binary ('{}')", executable_file_path.to_string_lossy());
        CBackend::emit_binary(&code, &executable_file_path)?;
    }

    if args.run {
        info!("Running '{}'", executable_file_path.to_string_lossy());

        let mut child = Command::new(&executable_file_path).spawn().expect("Failed to launch generated executable");
        let status = child.wait().expect("Failed to wait for child to finish execution");

        if fs::remove_file(&executable_file_path).is_err() {
            warn!("Failed to clean up temporary executable at '{}'", executable_file_path.to_string_lossy())
        }

        exit(status.code().unwrap_or(-1))
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
