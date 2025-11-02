use std::path::PathBuf;

use crate::args::error::ArgsError;

pub mod error;

const HELP_MESSAGE: &str = "\
usage: petal [flags] [options] <input>

flags:
    --help           Prints this message.
    --dump-ast       Prints the abstract syntax tree to stdout once parsed.
    --dump-bytecode  Prints the LLVM bytecode to stderr once parsed.

options:
    --output <path>  The path to write the final binary to.

args:
    <input>     The file to parse.
";

/// The arguments passed to the binary.
pub struct Args {
    /// Whether the help message should be printed.
    pub help: bool,

    /// Whether the raw AST representation should be dumped.
    pub dump_ast: bool,

    /// Whether the LLVM bytecode should be dumped.
    pub dump_bytecode: bool,

    /// The input file to read from.
    pub input: PathBuf,

    /// The output file to write the binary to.
    pub output: Option<PathBuf>,
}

impl Args {
    pub fn from_env() -> Result<Self, ArgsError> {
        let mut arguments = pico_args::Arguments::from_env();

        let args = Args {
            help: arguments.contains(["-h", "--help"]),

            dump_ast: arguments.contains("--dump-ast"),

            dump_bytecode: arguments.contains("--dump-bytecode"),

            output: arguments
                .opt_value_from_str(["-o", "--output"])
                .map_err(|_| ArgsError::cannot_parse_argument("output").into())?,

            input: arguments
                .free_from_str()
                .map_err(|_| ArgsError::missing_argument("input").into())?,
        };

        let remaining = arguments.finish();
        if !remaining.is_empty() {
            eprintln!("warn: unrecognized arguments: '{:?}'", remaining);
        }

        Ok(args)
    }

    pub fn print_help() {
        println!("{}", HELP_MESSAGE)
    }
}
