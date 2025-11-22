use std::fmt::Display;
use std::io::{self, stdout, Stdout, Write};
use std::ops::ControlFlow;
use termion::{
    clear, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

use crate::{commands::get_commands, eval::evaluate, parser::Parser, tokenizer::Tokenizer};

#[derive(Debug, PartialEq)]
enum CompletionState {
    NoCompletion,
    MultipleCompletion,
}

pub struct Shell {
    buffer: String,
    stdout: RawTerminal<Stdout>,
    completion_state: CompletionState,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            stdout: stdout().into_raw_mode().unwrap(),
            completion_state: CompletionState::NoCompletion,
        }
    }

    pub fn run(&mut self) {
        self.stdout.activate_raw_mode();

        loop {
            self.display(format!("$ {}", self.buffer));

            for key in io::stdin().keys().flatten() {
                match self.handle_key(key) {
                    ControlFlow::Continue(()) => continue,
                    ControlFlow::Break(()) => break,
                }
            }
        }
    }

    pub fn display(&mut self, s: impl Display) {
        write!(self.stdout, "{}", s).unwrap();
        self.stdout.flush().unwrap();
    }

    fn handle_key(&mut self, key: Key) -> ControlFlow<()> {
        match key {
            Key::Char('\t') => self.handle_autocompletion(),
            Key::Char('\n') => {
                self.handle_enter();
                ControlFlow::Break(())
            }
            Key::Char(c) => {
                write!(self.stdout, "{}", c).unwrap();
                self.stdout.flush().unwrap();

                self.buffer.push(c);
                ControlFlow::Continue(())
            }
            Key::Backspace if !self.buffer.is_empty() => {
                self.buffer.pop();
                write!(self.stdout, "{}{}", cursor::Left(1), clear::AfterCursor);
                self.stdout.flush().unwrap();
                ControlFlow::Continue(())
            }
            Key::Ctrl('c') => std::process::exit(0),
            _ => todo!(),
        }
    }

    fn handle_autocompletion(&mut self) -> ControlFlow<()> {
        let commands = get_commands();
        let mut completions: Vec<String> = commands
            .into_iter()
            .filter(|s| s.starts_with(&self.buffer))
            .collect();

        completions.sort();

        match completions.as_slice() {
            [] => {
                write!(self.stdout, "\x07").unwrap();
                self.stdout.flush().unwrap();
                ControlFlow::Continue(())
            }
            [completion] => {
                self.single_completion(completion.clone());
                ControlFlow::Continue(())
            }
            _ => match self.completion_state {
                CompletionState::NoCompletion => {
                    self.completion_state = CompletionState::MultipleCompletion;
                    write!(self.stdout, "\x07").unwrap();
                    self.stdout.flush().unwrap();
                    ControlFlow::Continue(())
                }
                CompletionState::MultipleCompletion => {
                    write!(self.stdout, "\n").unwrap();
                    write!(
                        self.stdout,
                        "{}",
                        cursor::Left(self.buffer.len() as u16 + 2),
                    )
                    .unwrap();
                    self.stdout.flush().unwrap();
                    self.stdout.suspend_raw_mode();
                    println!("{}", completions.join("  "));
                    self.stdout.activate_raw_mode();
                    ControlFlow::Break(())
                }
            },
        }
    }

    fn handle_enter(&mut self) {
        write!(self.stdout, "\n").unwrap();
        write!(
            self.stdout,
            "{}",
            cursor::Left(self.buffer.len() as u16 + 2)
        );

        self.stdout.flush().unwrap();

        let mut tokenizer = Tokenizer::new(&self.buffer);
        tokenizer.parse();
        let tokens = tokenizer.tokens();

        let mut parser = Parser::new(tokens);
        parser.parse();
        let ast = parser.ast();

        let output = evaluate(ast);

        self.stdout.suspend_raw_mode();
        print!("{}", String::from_utf8(output.stderr).unwrap());
        print!("{}", String::from_utf8(output.stdout).unwrap());
        self.stdout.activate_raw_mode();

        self.buffer.clear();
    }

    fn single_completion(&mut self, completion: String) {
        write!(
            self.stdout,
            "{}{}{} ",
            cursor::Left(self.buffer.len() as u16),
            clear::AfterCursor,
            completion,
        );
        self.stdout.flush();

        self.buffer = completion + " ";
    }
}
