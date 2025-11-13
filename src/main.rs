#[allow(unused_imports)]
use std::io::{self, Write};

mod commands;
mod eval;
mod parser;
mod tokenizer;

use tokenizer::{Token, Tokenizer};

use crate::{eval::evaluate, parser::Parser};

fn main() {
    let stdin = io::stdin();
    let mut buf = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();

        let tokens: Vec<Token> = Tokenizer::tokenize(buf.trim_end());
        let mut parser = Parser::new(tokens);
        parser.parse();
        let output = evaluate(parser.ast());
        print!("{}", String::from_utf8(output.stdout).unwrap());
        print!("{}", String::from_utf8(output.stderr).unwrap());
        buf.clear();
    }
}
