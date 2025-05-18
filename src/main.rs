use std::fs;

use ast::AST;
use codegen::Codegen;
use inkwell::context::Context;
use lexer::Lexer;
use typechecker::Typechecker;

pub mod ast;
pub mod codegen;
pub mod core;
pub mod lexer;
pub mod typechecker;

fn main() {
    let file_contents = fs::read_to_string("./examples/00_hello_world.petal").unwrap();

    let mut lexer = Lexer::from(&file_contents);
    let tokens = match lexer.parse() {
        Ok(value) => value,
        Err(error) => {
            return println!(
                "error(./examples/00_hello_world.petal:{}:{}): {}",
                error.location.line + 1,
                error.location.column + 1,
                error
            );
        }
    };

    let mut ast = AST::new(&tokens);
    let mut nodes = match ast.parse() {
        Ok(value) => value,
        Err(error) => {
            return match error.location {
                Some(location) => println!(
                    "error(./examples/00_hello_world.petal:{}:{}): {}",
                    location.line + 1,
                    location.column + 1,
                    error
                ),

                None => println!("error(./examples/00_hello_world.petal:unknown): {}", error),
            };
        }
    };

    let mut typechecker = Typechecker::new(&mut nodes);
    typechecker.check();

    let codegen_context = Context::create();
    let codegen = Codegen::new("00_hello_world", &codegen_context, &nodes);
    codegen.compile();
}
