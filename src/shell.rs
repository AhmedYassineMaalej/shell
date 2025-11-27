use std::fmt::Display;
use std::io::{self, Stdout, Write, stdout};
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::process::Stdio;
use termion::{
    clear, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

use crate::commands::Executable;
use crate::history::History;
use crate::{commands::get_commands, parser::Parser, tokenizer::Tokenizer};

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
    history: History,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            stdout: stdout().into_raw_mode().expect("failed to set raw mode"),
            raw_mode: true,
            completion_state: CompletionState::None,
            history: History::new(),
        }
    }

    pub fn run(&mut self) {
        self.read_history_file();
        self.set_raw_mode(true);

        loop {
            self.display(format!("$ {}", self.buffer));

            for key in io::stdin().keys().flatten() {
                if let ControlFlow::Break(()) = self.handle_key(key) {
                    break;
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
            Key::Up => {
                self.handle_up_arrow();
                ControlFlow::Continue(())
            }
            Key::Down => {
                self.handle_down_arrow();
                ControlFlow::Continue(())
            }
            k => {
                self.set_raw_mode(false);
                todo!("{:?}", k);
            }
        }
    }

    fn handle_up_arrow(&mut self) {
        let Some(command) = self.history.prev().cloned() else {
            return;
        };

        if self.buffer.is_empty() {
            self.display(&command);
        } else {
            self.display(format!(
                "{}{}{}",
                cursor::Left(self.buffer.len().try_into().unwrap()),
                clear::AfterCursor,
                command,
            ));
        }

        self.buffer = command;
    }

    fn handle_down_arrow(&mut self) {
        let Some(command) = self.history.next().cloned() else {
            return;
        };

        if self.buffer.is_empty() {
            self.display(&command);
        } else {
            self.display(format!(
                "{}{}{}",
                cursor::Left(self.buffer.len().try_into().unwrap()),
                clear::AfterCursor,
                command,
            ));
        }

        self.buffer = command;
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

        self.multiple_completions(&completions)
    }

    fn prefix_completion(&mut self, prefix: &str) {
        self.display(format!(
            "{}{}{}",
            cursor::Left(self.buffer.len().try_into().unwrap()),
            clear::AfterCursor,
            prefix,
        ));

        self.buffer = String::from(prefix);
    }

    fn multiple_completions(&mut self, completions: &[String]) -> ControlFlow<()> {
        if self.completion_state == CompletionState::None {
            let prefix = completions_prefix(completions);

            if !prefix.is_empty() && prefix != self.buffer {
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
        self.display(format!(
            "\n{}",
            cursor::Left(u16::try_from(self.buffer.len()).unwrap() + 2)
        ));
    }

    fn handle_enter(&mut self) {
        self.history.add(self.buffer.clone());

        self.newline();

        let tokens = Tokenizer::tokenize(&self.buffer);

        let mut parser = Parser::new(tokens);
        parser.parse();
        let ast = parser.ast();

        self.set_raw_mode(false);
        ast.execute(self, Stdio::inherit(), io::stdout(), io::stderr())
            .map(|mut child| child.wait());

        self.buffer.clear();
    }

    fn single_completion(&mut self, completion: String) {
        self.display(format!(
            "{}{}{} ",
            cursor::Left(self.buffer.len().try_into().unwrap()),
            clear::AfterCursor,
            completion,
        ));

        self.buffer = completion + " ";
    }

    pub fn history(&mut self) -> &mut History {
        &mut self.history
    }

    fn read_history_file(&mut self) {
        let Ok(path) = std::env::var("HISTFILE") else {
            return;
        };

        self.history.read_from_file(PathBuf::from(path));
    }
}

fn completions_prefix(completions: &[String]) -> &str {
    completions
        .iter()
        .map(String::as_str)
        .reduce(|a, b| common_prefix(a, b))
        .unwrap()
}

fn common_prefix<'a>(word1: &'a str, word2: &'a str) -> &'a str {
    let i = word1
        .chars()
        .zip(word2.chars())
        .take_while(|(c1, c2)| c1 == c2)
        .count();

    &word1[..i]
}
