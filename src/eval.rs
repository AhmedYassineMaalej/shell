use std::fs;

use crate::{
    commands::{Command, CommandContext, CommandOutput},
    parser::{Expr, Stream},
};

pub fn evaluate(ast: Expr) -> CommandOutput {
    match ast {
        Expr::Command { name, args } => {
            let command = CommandContext::new(Command::from(name.as_str()), args);
            command.execute()
        }
        Expr::Redirect { src, stream, dest } => redirect(src, stream, dest),
    }
}

fn redirect(src: Box<Expr>, stream: Stream, dest: String) -> CommandOutput {
    let src = evaluate(*src);

    match stream {
        Stream::Stdin => todo!(),
        Stream::Stdout => fs::write(dest, &src.stdout),
        Stream::Stderr => todo!(),
    };

    CommandOutput::new()
}
