#[allow(unused_imports)]
use std::io::{self, Write};

mod autocomplete;
mod commands;
mod eval;
mod parser;
mod tokenizer;

use rustyline::{history::DefaultHistory, Editor};

use tokenizer::{Token, Tokenizer};

use crate::{autocomplete::CmdHelper, eval::evaluate, parser::Parser};

fn main() {
    let mut editor: Editor<CmdHelper, DefaultHistory> = Editor::new().unwrap();
    editor.set_helper(Some(CmdHelper));

    loop {
        let input = editor.readline("$ ");

        match input {
            Ok(line) => {
                if &line == "exit" {
                    break;
                }

                let tokens: Vec<Token> = Tokenizer::tokenize(&line.trim_end());
                let mut parser = Parser::new(tokens);
                parser.parse();
                let output = evaluate(parser.ast());
                print!("{}", String::from_utf8(output.stderr).unwrap());
                print!("{}", String::from_utf8(output.stdout).unwrap());
            }
            Err(_) => break,
        }
    }
}
