use std::process;

use colored::Colorize;
use petal_ast::{ASTParser, visitor::dump_visitor::DumpASTVisitor};
use petal_core::{error::Error, module::Module};
use petal_lexer::Lexer;
use petal_llvm_codegen::{LLVMCodegen, LLVMCodegenContext};
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

    let mut ast_parser = ASTParser::new(token_stream);

    let mut statement_stream = match ast_parser.parse() {
        Ok(value) => value,
        Err(error) => {
            print_error(&module, error);
            process::exit(1);
        }
    };

    if let Err(error) = statement_stream.visit(&mut Typechecker::new(module.string_intern_pool.as_ref())) {
        print_error(&module, error);
        process::exit(1);
    }

    if args.dump_ast {
        if let Err(error) = statement_stream.visit(&mut DumpASTVisitor::new()) {
            print_error(&module, error);
            process::exit(1);
        }
    }

    let codegen_context = LLVMCodegenContext::new();
    let mut codegen = LLVMCodegen::new(&codegen_context);

    if let Err(error) = statement_stream.visit(&mut codegen) {
        print_error(&module, error);
        process::exit(1);
    }
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
