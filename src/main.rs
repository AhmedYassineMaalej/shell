#[allow(unused_imports)]
use std::io::{self, Write};
use std::io::{stdin, stdout};

use termion::{
    clear,
    cursor::{self, DetectCursorPos},
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
};

use crate::{commands::get_commands, eval::evaluate, parser::Parser, tokenizer::Tokenizer};

mod autocomplete;
mod commands;
mod eval;
mod parser;
mod tokenizer;

fn main() {
    let mut buffer = String::new();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut last_key: Option<Key> = None;

    loop {
        stdout.activate_raw_mode();

        write!(stdout, "$ ").unwrap();
        stdout.flush().unwrap();

        for key in io::stdin().keys().flatten() {
            match key {
                Key::Char('\t') => {
                    let commands = get_commands();
                    let mut completions: Vec<String> = commands
                        .into_iter()
                        .filter(|s| s.starts_with(&buffer))
                        .collect();

                    completions.sort();

                    if completions.len() == 1 {
                        write!(
                            stdout,
                            "{}{}{}",
                            cursor::Left(buffer.len() as u16),
                            clear::AfterCursor,
                            completions[0]
                        );
                        stdout.flush();

                        buffer = completions.into_iter().next().unwrap();
                    } else if completions.len() > 1 && last_key == Some(Key::Char('\t')) {
                        write!(stdout, "\n").unwrap();
                        write!(stdout, "{}", cursor::Left(buffer.len() as u16 + 2),).unwrap();
                        stdout.flush().unwrap();
                        stdout.suspend_raw_mode();
                        println!("{}", completions.join("  "));
                        stdout.activate_raw_mode();
                        buffer.clear();
                        break;
                    } else if completions.len() > 1 {
                        write!(stdout, "\x07").unwrap();
                        stdout.flush().unwrap();
                    }
                }
                Key::Char('\n') => {
                    write!(stdout, "\n").unwrap();
                    stdout.flush().unwrap();
                    break;
                }
                Key::Char(c) => {
                    write!(stdout, "{}", c).unwrap();
                    stdout.flush().unwrap();

                    buffer.push(c);
                }
                Key::Backspace if !buffer.is_empty() => {
                    buffer.pop();
                    write!(stdout, "{}{}", cursor::Left(1), clear::AfterCursor);
                    stdout.flush().unwrap();
                }
                Key::Ctrl('c') => break,
                _ => todo!(),
            }

            last_key = Some(key);
        }

        write!(stdout, "{}", cursor::Left(buffer.len() as u16 + 2),).unwrap();
        stdout.suspend_raw_mode().unwrap();

        if buffer.is_empty() {
            continue;
        }

        let mut tokenizer = Tokenizer::new(&buffer);
        tokenizer.parse();
        let tokens = tokenizer.tokens();

        let mut parser = Parser::new(tokens);
        parser.parse();
        let ast = parser.ast();

        let output = evaluate(ast);

        print!("{}", String::from_utf8(output.stderr).unwrap());
        print!("{}", String::from_utf8(output.stdout).unwrap());

        buffer.clear();
    }
}
