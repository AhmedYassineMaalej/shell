use std::fmt::Display;
use std::io::{self, Stdout, Write, stdout};
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
    None,
    Multiple,
}

pub struct Shell {
    buffer: String,
    stdout: RawTerminal<Stdout>,
    raw_mode: bool,
    completion_state: CompletionState,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            stdout: stdout().into_raw_mode().expect("failed to set raw mode"),
            raw_mode: true,
            completion_state: CompletionState::None,
        }
    }

    pub fn run(&mut self) {
        self.set_raw_mode(true);

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

    fn display(&mut self, s: impl Display) {
        self.set_raw_mode(true);
        write!(self.stdout, "{s}").expect("failed to write to raw stdout");
        self.stdout.flush().expect("failed to flush stdout");
    }

    fn bell(&mut self) {
        self.display("\x07");
    }

    fn set_raw_mode(&mut self, raw_mode: bool) {
        if self.raw_mode == raw_mode {
            return;
        }
        self.raw_mode = raw_mode;

        if raw_mode {
            self.stdout
                .activate_raw_mode()
                .expect("failed to activated raw mode");
        } else {
            self.stdout
                .suspend_raw_mode()
                .expect("failed to suspend raw mode");
        }
    }

    fn handle_key(&mut self, key: Key) -> ControlFlow<()> {
        match key {
            Key::Char('\t') => self.handle_autocompletion(),
            Key::Char('\n') => {
                self.handle_enter();
                ControlFlow::Break(())
            }
            Key::Char(c) => {
                self.display(c);
                self.buffer.push(c);
                ControlFlow::Continue(())
            }
            Key::Backspace if !self.buffer.is_empty() => {
                self.buffer.pop();
                self.display(format!("{}{}", cursor::Left(1), clear::AfterCursor));
                ControlFlow::Continue(())
            }
            Key::Ctrl('c') => std::process::exit(0),
            _ => {
                self.set_raw_mode(false);
                todo!();
            }
        }
    }

    fn handle_autocompletion(&mut self) -> ControlFlow<()> {
        let commands = get_commands();
        let mut completions: Vec<String> = commands
            .into_iter()
            .filter(|s| s.starts_with(&self.buffer) && s != &self.buffer)
            .collect();

        completions.sort();

        if completions.is_empty() {
            self.bell();
            return ControlFlow::Continue(());
        }

        if completions.len() == 1 {
            let completion = completions.into_iter().next().unwrap();
            self.single_completion(completion);
            return ControlFlow::Continue(());
        }

        self.multiple_completions(completions)
    }

    fn prefix_completion(&mut self, prefix: &str) {
        self.display(format!(
            "{}{}{}",
            cursor::Left(self.buffer.len() as u16),
            clear::AfterCursor,
            prefix,
        ));

        self.buffer = String::from(prefix);
    }

    fn multiple_completions(&mut self, completions: Vec<String>) -> ControlFlow<()> {
        if self.completion_state == CompletionState::None {
            let prefix = completions_prefix(&completions);

            if !prefix.is_empty() && prefix != &self.buffer {
                self.prefix_completion(prefix);
            } else {
                self.completion_state = CompletionState::Multiple;
                self.bell();
            }

            return ControlFlow::Continue(());
        }

        self.newline();

        self.set_raw_mode(false);
        println!("{}", completions.join("  "));

        ControlFlow::Break(())
    }

    fn newline(&mut self) {
        self.display(format!("\n{}", cursor::Left(self.buffer.len() as u16 + 2)));
    }

    fn handle_enter(&mut self) {
        self.newline();

        let mut tokenizer = Tokenizer::new(&self.buffer);
        tokenizer.parse();
        let tokens = tokenizer.tokens();

        let mut parser = Parser::new(tokens);
        parser.parse();
        let ast = parser.ast();

        self.set_raw_mode(false);
        evaluate(ast);

        self.buffer.clear();
    }

    fn single_completion(&mut self, completion: String) {
        self.display(format!(
            "{}{}{} ",
            cursor::Left(self.buffer.len() as u16),
            clear::AfterCursor,
            completion,
        ));

        self.buffer = completion + " ";
    }
}

fn completions_prefix(completions: &Vec<String>) -> &str {
    completions
        .iter()
        .map(|s| s.as_str())
        .reduce(|a, b| common_prefix(a, b))
        .unwrap()
}

fn common_prefix<'a>(word1: &'a str, word2: &'a str) -> &'a str {
    let mut i = word1
        .chars()
        .zip(word2.chars())
        .take_while(|(c1, c2)| c1 == c2)
        .count();

    &word1[..i]
}
