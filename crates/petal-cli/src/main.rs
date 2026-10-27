use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode},
};

use petal_codegen::codegen;

use crate::diagnostic::print_diagnostics;

mod diagnostic;

fn main() -> ExitCode {
    let mut arguments = env::args();

    let program_name = arguments.next().unwrap_or("petal".into());

    let Some(path) = arguments.next() else {
        eprintln!("usage: {program_name} <path> [-o <output>] [--dump-ast] [--dump-tokens]");
        return ExitCode::FAILURE;
    };

    let mut dump_tokens = false;
    let mut dump_ast = false;
    let mut output_path = None;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--dump-tokens" => dump_tokens = true,
            "--dump-ast" => dump_ast = true,
            "-o" => {
                let Some(path) = arguments.next() else {
                    eprintln!("expected a path after '-o'");
                    return ExitCode::FAILURE;
                };

                output_path = Some(PathBuf::from(path));
            }
            _ => {
                eprintln!("unknown argument: '{argument}'");
                return ExitCode::FAILURE;
            }
        }
    }

    let source = fs::read_to_string(&path).expect("failed to read source file");
    let tokens = petal_lexer::tokenize(&source);

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

    let object = codegen(&check_result, &parse_result.definitions);

    let source_path = PathBuf::from(path);
    let binary_file_name = output_path.unwrap_or_else(|| source_path.with_extension(""));

    // Without an extension on the source file, the default output path would overwrite the source file.
    if binary_file_name == source_path {
        eprintln!(
            "refusing to overwrite source file '{}', pass '-o <output>' to choose an output path",
            source_path.display()
        );
        return ExitCode::FAILURE;
    }

    let mut object_file_name = binary_file_name.clone().into_os_string();
    object_file_name.push(".o");
    let object_file_name = PathBuf::from(object_file_name);

    fs::write(&object_file_name, object).expect("failed to write object file");

    let link_status = Command::new("cc")
        .arg(&object_file_name)
        .arg("-o")
        .arg(&binary_file_name)
        .status()
        .expect("failed to spawn cc to link object");

    if !link_status.success() {
        eprintln!("failed to link {}", object_file_name.display());
        return ExitCode::FAILURE;
    }

    let _ = fs::remove_file(object_file_name);

    println!(
        "Compiled source file '{}' to executable '{}'",
        source_path.display(),
        binary_file_name.display()
    );

    ExitCode::SUCCESS
}
