use std::{
    fs::OpenOptions,
    io::{self, stderr},
    os::unix::process::CommandExt,
    process::Stdio,
};

use crate::{
    commands::{Command, Executable, find_path},
    parser::{Expr, Stream},
};

pub fn evaluate(ast: Expr) {
    match ast {
        Expr::Command { name, args } => {
            let command = Command::new(name, args);
            if let Some(mut child) = command.execute(Stdio::inherit(), io::stdout(), io::stderr()) {
                child.wait();
            }
        }
        Expr::Redirect { src, stream, dest } => redirect(src, stream, dest),
        Expr::Append { src, stream, dest } => append(src, stream, dest),
        Expr::Pipe { src, dest } => pipe(src, dest),
    }
}

fn redirect(src: Box<Expr>, stream: Stream, dest: String) {
    let Expr::Command { name, args } = *src else {
        panic!("expected command before pipe");
    };

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest)
        .unwrap();

    let command = Command::new(name, args);

    match stream {
        Stream::Stdin => todo!(),
        Stream::Stdout => {
            if let Some(mut child) = command.execute(Stdio::inherit(), file, io::stderr()) {
                child.wait();
            }
        }
        Stream::Stderr => {
            if let Some(mut child) = command.execute(Stdio::inherit(), io::stdout(), file) {
                child.wait();
            }
        }
    };
}

fn append(src: Box<Expr>, stream: Stream, dest: String) {
    let Expr::Command { name, args } = *src else {
        panic!("expected command before pipe");
    };

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(dest)
        .unwrap();

    let command = Command::new(name, args);

    match stream {
        Stream::Stdin => todo!(),
        Stream::Stdout => command.execute(Stdio::inherit(), file, io::stderr()),
        Stream::Stderr => command.execute(Stdio::inherit(), io::stdout(), file),
    };
}

fn pipe(src: Box<Expr>, dest: Box<Expr>) {
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

    let (pipe_reader, pipe_writer) = std::io::pipe().unwrap();

    let cmd1 = Command::new(src_name, src_args);
    let child1 = cmd1.execute(Stdio::inherit(), pipe_writer, io::stderr());

    let cmd2 = Command::new(dest_name, dest_args);
    let child2 = cmd2.execute(pipe_reader, io::stdout(), io::stderr());

    if let Some(mut child) = child2 {
        child.wait();
    }

    if let Some(mut child) = child1 {
        child.wait();
    }
}
