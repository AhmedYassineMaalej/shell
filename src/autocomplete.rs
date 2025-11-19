use rustyline::{completion::Completer, Context, Helper, Result};

use crate::commands::get_commands;

pub struct CmdHelper;

impl rustyline::validate::Validator for CmdHelper {}

impl rustyline::highlight::Highlighter for CmdHelper {}

impl rustyline::hint::Hinter for CmdHelper {
    type Hint = String;
}

impl Completer for CmdHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        let commands = get_commands();
        let mut completions = Vec::new();

        for mut command in commands {
            if command.starts_with(line) {
                command.push(' ');
                completions.push(command);
            }
        }

        Ok((0, completions))
    }
}

impl Helper for CmdHelper {}
