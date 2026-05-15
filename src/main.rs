use std::{
    env::{
        self,
        current_dir,
    },
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
    ast::statement::{
        StatementKind,
        import::Import,
    },
    backend::c::CBackend,
    core::error::Error,
    module::{
        ModuleError,
        ParsedModule,
    },
    module_registry::{
        ModuleId,
        ModuleRegistry,
    },
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

fn handle_module_import(
    parsed_modules: &mut Vec<ParsedModule>,
    module_registry: &mut ModuleRegistry,
    current_module: &ParsedModule,
    current_path: &PathBuf,
    import: &Import,
) -> Result<(), Box<dyn Error>> {
    // If the name of the imported module is `stdlib`, it must be next to the compiler's current working
    // directory.
    //
    // Otherwise, we could either be:
    // - Importing a directory module
    // - Importing a file module
    let mut path = if import.name == "stdlib" {
        // TODO: `PETAL_STDLIB_PATH` environment variable?
        current_dir().map_err(|e| ModuleError::IOError { path: current_path.clone(), error: e })?.join("stdlib")
    } else {
        current_path.with_file_name(&import.name)
    };

    // If something exists for the module path, then we can assume it is the directory (as we have not appended)
    // the `.petal` extension yet.
    let path_exists = fs::exists(&path).map_err(|e| ModuleError::IOError { path: path.clone(), error: e })?;
    let path_is_dir = fs::metadata(&path).map(|it| it.is_dir()).unwrap_or_default();
    if path_exists && path_is_dir {
        trace!(
            "Module {} imports '{}', which resolves to path '{}', which is a directory module",
            current_module.id,
            import.name,
            path.display()
        );

        import_directory_module(parsed_modules, module_registry, &path)?;
    } else {
        // A directory does not exist with the provided name. Our last resort is to try to find a `.petal` file with
        // the same name.
        path.set_extension("petal");

        trace!(
            "Module {} imports '{}', which resolves to path '{}', which is a single-file module",
            current_module.id,
            import.name,
            path.display()
        );

        let module_id = create_and_parse_module(parsed_modules, module_registry, &path)?;
        trace!("Module at path '{}' resolves to module ID {}", path.display(), module_id);
    }

    Ok(())
}

fn import_directory_module(
    parsed_modules: &mut Vec<ParsedModule>,
    module_registry: &mut ModuleRegistry,
    directory_path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    // The caller ensures that the directory exists, so we just need to read it & iterate over its children.
    for entry in
        fs::read_dir(&directory_path).map_err(|e| ModuleError::IOError { path: directory_path.clone(), error: e })?
    {
        let entry = entry.map_err(|e| ModuleError::IOError { path: directory_path.clone(), error: e })?;
        if entry.path().extension().map(|it| it != "petal").unwrap_or_default() {
            trace!(
                "Ignoring directory entry '{}' for directory import as it does not contain the petal extension",
                entry.path().display()
            );

            continue;
        }

        if entry.metadata().map_err(|e| ModuleError::IOError { path: directory_path.clone(), error: e })?.is_dir() {
            warn!(
                "Directory entry '{}' is being ignored as it is a subdirectory of a module that is being imported",
                entry.path().display()
            );
            continue;
        }

        // TODO: Nested directories?

        let module_id = create_and_parse_module(parsed_modules, module_registry, &entry.path())?;
        trace!(
            "  - Module at relative path '{}' resolves to module ID {}",
            entry.path().strip_prefix(directory_path).unwrap().display(),
            module_id
        );
    }

    Ok(())
}

fn create_and_parse_module(
    parsed_modules: &mut Vec<ParsedModule>,
    module_registry: &mut ModuleRegistry,
    file_path: &PathBuf,
) -> Result<ModuleId, Box<dyn Error>> {
    let (module_id, already_created) = module_registry.create_module(file_path.clone())?;
    if already_created {
        trace!("Module at path '{}' has already been registered (ID = {})", file_path.display(), module_id);
        return Ok(module_id);
    }

    let parsed_module = module_registry.get_module(module_id).parse()?;

    for statement in &parsed_module.ast {
        // If this is an import statement, then we must be able to find a module with the imported name in the
        // same directory.
        if let StatementKind::Import(import) = &statement.kind {
            handle_module_import(parsed_modules, module_registry, &parsed_module, &file_path, import)?;
        }
    }

    parsed_modules.push(parsed_module);
    Ok(module_id)
}

fn main_impl(mut args: Args, module_registry: &mut ModuleRegistry) -> Result<(), Box<dyn Error>> {
    if args.no_emit_binary && args.run {
        warn!("--no-emit-binary and --run provided as arguments, this is not supported. Ignoring --no-emit-binary!");
        args.no_emit_binary = false;
    }

    info!("Parsing modules");

    let mut parsed_modules: Vec<ParsedModule> = Vec::new();

    // Before parsing any user code, we must import the prelude module.
    {
        let mut path = current_dir().expect("current_dir");
        path.push("prelude");

        trace!("Importing prelude module from path '{}'", path.display());
        import_directory_module(&mut parsed_modules, module_registry, &path)?;
    }

    for file_path_string in &args.input {
        let file_path = PathBuf::from(file_path_string);
        let module_id = create_and_parse_module(&mut parsed_modules, module_registry, &file_path)?;
        trace!("Module {} created from compiler arguments, resolves to path '{}'", module_id, file_path.display());
    }

    info!("Checking types");

    let checked_program = Typechecker::default().check(parsed_modules)?;

    info!("Generating code");

    let code = CBackend::new(
        checked_program.builtin_types,
        checked_program.declared_types,
        checked_program.enums,
        checked_program.functions,
        checked_program.structures,
        checked_program.specialized_functions,
        checked_program.specialized_structures,
        checked_program.synthetic_types,
    )
    .emit_code(&checked_program.modules)?;

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
