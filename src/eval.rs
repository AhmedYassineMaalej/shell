use std::{
    fs::{self, OpenOptions},
    io::Write,
};

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
        Expr::Append { src, stream, dest } => redirect(src, stream, dest),
    }
}

fn redirect(src: Box<Expr>, stream: Stream, dest: String) -> CommandOutput {
    let src = evaluate(*src);

    match stream {
        Stream::Stdin => todo!(),
        Stream::Stdout => {
            fs::write(dest, &src.stdout);
            CommandOutput {
                stdout: Vec::new(),
                stderr: src.stderr,
                success: src.success,
            }
        }
        Stream::Stderr => {
            fs::write(dest, &src.stderr);
            CommandOutput {
                stdout: src.stdout,
                stderr: Vec::new(),
                success: src.success,
            }
        }
    }
}

fn append(src: Box<Expr>, stream: Stream, dest: String) -> CommandOutput {
    let src = evaluate(*src);

    match stream {
        Stream::Stdin => todo!(),
        Stream::Stdout => {
            append_to_file(dest, &src.stdout);
            CommandOutput {
                stdout: Vec::new(),
                stderr: src.stderr,
                success: src.success,
            }
        }
        Stream::Stderr => {
            append_to_file(dest, &src.stderr);
            CommandOutput {
                stdout: src.stdout,
                stderr: Vec::new(),
                success: src.success,
            }
        }
    }
}

fn append_to_file(file: String, content: &[u8]) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file)
        .unwrap();

    file.write_all(content);
}
