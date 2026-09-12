use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let mut arguments = env::args();

    let program_name = arguments.next().unwrap_or("petal".into());

    let Some(path) = arguments.next() else {
        eprintln!("usage: {program_name} <path> [--dump-tokens]");
        return ExitCode::FAILURE;
    };

    let source = fs::read_to_string(path).expect("failed to read source file");
    let tokens = petal_lexer::tokenize(&source);

    let dump_tokens = arguments.next().is_some_and(|it| it == "--dump-tokens");
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
    }

    ExitCode::SUCCESS
}
