use std::fs;

use lexer::Lexer;

pub mod lexer;

fn main() {
    let file_contents = fs::read_to_string("./examples/00_hello_world.petal").unwrap();

    let mut lexer = Lexer::from(&file_contents);
    let tokens = match lexer.parse() {
        Ok(value) => value,
        Err(error) => return println!("ERROR: {}", error),
    };

    println!("Tokens: {:?}", tokens);
}
