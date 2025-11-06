use std::{
    fs,
    process::{self, Command},
};

use colored::Colorize;
use petal_ast::{ASTParser, visitor::dump_visitor::DumpASTVisitor};
use petal_codegen_driver::{Driver, options::DriverOptions};
use petal_core::{error::Error, module::Module};
use petal_lexer::Lexer;
use petal_llvm_codegen::LLVMCodegen;
use petal_typechecker::Typechecker;

use crate::args::Args;

pub mod args;

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

    let mut module = match Module::new(args.input) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error);
            process::exit(-1);
        }
    };

    let mut lexer = Lexer::new(module.string_intern_pool.as_mut(), &module.contents);

    let token_stream = match lexer.get_stream() {
        Ok(value) => value,
        Err(error) => {
            print_error(&module, error);
            process::exit(1);
        }
    };

    let mut ast_parser = ASTParser::new(token_stream, &mut module.type_pool);

    let mut statement_stream = match ast_parser.parse() {
        Ok(value) => value,
        Err(error) => {
            print_error(&module, error);
            process::exit(1);
        }
    };

    if let Err(error) = statement_stream.visit(&mut Typechecker::new(
        &mut module.type_pool,
        module.string_intern_pool.as_ref(),
    )) {
        print_error(&module, error);
        process::exit(1);
    }

    if args.dump_ast {
        if let Err(error) = statement_stream.visit(&mut DumpASTVisitor::new()) {
            print_error(&module, error);
            process::exit(1);
        }
    }

    let mut codegen = LLVMCodegen::new(
        DriverOptions {
            module_name: module.name(),
            dump_bytecode: args.dump_bytecode,
        },
        &module.type_pool,
        module.string_intern_pool.as_ref(),
    );

    if let Err(error) = statement_stream.visit(&mut codegen) {
        print_error(&module, error);
        process::exit(1);
    }

    let object_file_path = match codegen.compile_to_object() {
        Ok(value) => value,
        Err(error) => {
            eprintln!(
                "{}: {}",
                String::from("error").red().bold(),
                format!("{}", error).bold(),
            );

            process::exit(1);
        }
    };

    if let Some(output) = args.output {
        let output_parent_directory = match output.parent() {
            Some(value) => value,
            None => {
                eprintln!("error: could not get parent of {}", output.display());
                process::exit(-1);
            }
        };

        // If the parent directory does not exist yet, we must try to create it.
        if !fs::exists(output_parent_directory).unwrap_or(false) {
            if let Err(error) = fs::create_dir(output_parent_directory) {
                eprintln!(
                    "error: failed to create directory '{}': {}",
                    output_parent_directory.display(),
                    error
                )
            }
        }

        let result = Command::new("cc")
            .arg("-o")
            .arg(&output)
            .arg(&object_file_path)
            .status()
            .expect("Failed to invoke `cc` to link the final executable!");

        if result.success() {
            println!(
                "{}: executable written to {}",
                format!("success").bright_green().bold(),
                output.display()
            )
        } else {
            eprintln!(
                "{}: {}",
                String::from("error").red().bold(),
                format!("linking failed, result: {:?}", result).bold(),
            );
        }
    } else if !args.dump_ast && !args.dump_bytecode {
        println!(
            "{}: not emitting anything as no `output`, `dump-ast` or `dump-bytecode` options were passed",
            format!("warn").bright_yellow().bold()
        )
    }

    // The OS should clean it up eventually as it should be a temporary file.
    let _ = fs::remove_file(&object_file_path);
}

fn print_error(module: &Module, error: Error) {
    let (error_line_number, error_column_number) = error.span.get_line_and_column(&module.contents);

    eprintln!(
        "{}: {}",
        format!(
            "{}({}:{}:{})",
            String::from("error").red().bold(),
            module.input.display(),
            error_line_number,
            error_column_number
        )
        .white(),
        format!("{}", error).bold(),
    );

    // In order to print some more context, we can attempt to print the line before the one that the error occurred on.
    if error_line_number > 2
        && let Some(line) = module.contents.lines().nth(error_line_number - 2)
    {
        eprintln!("{} {}", format!("{} |", error_line_number - 1).white(), line.white());
    }

    // We can then print the line that the error was found on, with some carets underneath to indicate the token that
    // caused the error.
    if let Some(line) = module.contents.lines().nth(error_line_number - 1) {
        eprintln!("{} {}", format!("{} |", error_line_number).white(), line.bright_white());

        eprintln!(
            "    {}{}",
            " ".repeat(error_column_number - 1),
            "^".repeat(error.span.length()).red().bold()
        );
    }
}
