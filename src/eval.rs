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
        Expr::Append { src, stream, dest } => append(src, stream, dest),
        Expr::Pipe { src, dest } => pipe(src, dest),
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

fn pipe(src: Box<Expr>, dest: Box<Expr>) -> CommandOutput {
    let Expr::Command {
        name: src_name,
        args: src_args,
    } = *src
    else {
        panic!("expected command after pipe");
    };

    let Expr::Command {
        name: dest_name,
        args: dest_args,
    } = *dest
    else {
        panic!("expected command after pipe");
    };

    CommandContext::execute_binary_piped(src_name, src_args, dest_name, dest_args)
}
