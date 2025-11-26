#![warn(clippy::pedantic)]

use crate::shell::Shell;

mod commands;
mod eval;
mod parser;
mod shell;
mod tokenizer;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
