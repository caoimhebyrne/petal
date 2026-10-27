use std::{env, fs, process::ExitCode};

use crate::diagnostic::print_diagnostics;

mod diagnostic;

fn main() -> ExitCode {
    let mut arguments = env::args();

    let program_name = arguments.next().unwrap_or("petal".into());

    let Some(path) = arguments.next() else {
        eprintln!("usage: {program_name} <path> [--dump-ast] [--dump-tokens]");
        return ExitCode::FAILURE;
    };

    let dump_argument = arguments.next();

    let source = fs::read_to_string(&path).expect("failed to read source file");
    let tokens = petal_lexer::tokenize(&source);

    let dump_tokens = dump_argument.as_ref().is_some_and(|it| it == "--dump-tokens");
    if dump_tokens {
        for token in tokens {
            println!(
                "{:<23} {:>6}..{:<6} {:?}",
                format!("{:?}", token.kind),
                token.span.start(),
                token.span.end(),
                token.span.slice(&source)
            );
        }

        return ExitCode::SUCCESS;
    }

    let parse_result = petal_ast::parse(&source, tokens);

    let dump_ast = dump_argument.as_ref().is_some_and(|it| it == "--dump-ast");
    if dump_ast {
        for definition in &parse_result.definitions {
            println!("{definition:#?}");
        }
    }

    print_diagnostics(&path, &parse_result.diagnostics);
    if !parse_result.can_continue() {
        return ExitCode::FAILURE;
    }

    let check_result = petal_typecheck::check(&parse_result);

    print_diagnostics(&path, &check_result.diagnostics);
    if !check_result.can_continue() {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
