use std::{
    fs,
    process::{self, Command, exit},
};

use petal_codegen_driver::{Driver, options::DriverOptions};
use petal_llvm_codegen::LLVMCodegen;
use petal_typechecker::Typechecker;

use crate::{args::Args, compiler_state::CompilerState, module::Module};

pub mod args;
pub mod compiler_state;
pub mod module;

/// This is the entrypoint for the Petal compiler.
fn main() {
    let args = match Args::from_env() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error);

            Args::print_help();
            process::exit(-1);
        }
    };

    if args.help {
        Args::print_help();
        return;
    }

    let mut compiler_state = CompilerState::new();

    // The first module we compile is the main module. This module can then import other modules.
    let main_module = match Module::new(args.input.clone()) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: failed to initialize main module: {}", error);
            process::exit(-1);
        }
    };

    // We can then resolve all modules involved with the main module.
    let mut resolved_modules = match main_module.resolve(&mut compiler_state) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error);
            process::exit(-1);
        }
    };

    println!(
        "info: resolved {} modules from {}",
        resolved_modules.len(),
        args.input.display()
    );

    let mut typechecker = Typechecker::new(
        &mut compiler_state.type_pool,
        compiler_state.string_intern_pool.as_mut(),
    );

    if let Err(error) = typechecker.check_modules(&mut resolved_modules) {
        eprintln!("error: {}", error);
        process::exit(-1);
    }

    println!("info: all modules passed type checking successfully");

    let mut codegen = LLVMCodegen::new(
        DriverOptions {
            module_name: args
                .input
                .with_extension("")
                .file_name()
                .map(|it| it.to_string_lossy().to_string())
                .unwrap_or("unnamed module".to_owned()),

            dump_bytecode: args.dump_bytecode,
        },
        &compiler_state.type_pool,
        compiler_state.string_intern_pool.as_ref(),
    );

    if let Err(error) = codegen.visit_modules(&resolved_modules) {
        eprintln!("error: {}", error);
        process::exit(-1);
    }

    let object_path = match codegen.compile_to_object() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error);
            process::exit(-1);
        }
    };

    if let Some(output_path) = args.output {
        let command = Command::new("cc")
            .arg("-o")
            .arg(&output_path)
            .arg(&object_path)
            .status();

        let status = match command {
            Ok(value) => value,
            Err(error) => {
                eprintln!("error: failed to link final executable: {}", error);
                process::exit(-1);
            }
        };

        if status.success() {
            println!("success: executable written to {}", output_path.display());
        } else {
            eprintln!("error: linker exited with non-zero status code: {}", status);
            exit(-1);
        }
    } else if !args.dump_ast && !args.dump_bytecode {
        println!(
            "warn: compilation was successful, but no output was produced as no output-emitting arguments were passed"
        )
    }

    let _ = fs::remove_file(object_path);
}
