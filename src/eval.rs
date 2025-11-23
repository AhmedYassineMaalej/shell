use std::{fs::OpenOptions, io, os::unix::process::CommandExt, process::Stdio};

use crate::{
    commands::{Command, Executable, find_path},
    parser::{Expr, Stream},
};

pub fn evaluate(ast: Expr) {
    match ast {
        Expr::Command { name, args } => {
            let command = Command::new(name, args);
            command.execute(Stdio::inherit(), io::stdout(), io::stderr());
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

    let file = OpenOptions::new().truncate(true).open(dest).unwrap();

    let command = Command::new(name, args);

    match stream {
        Stream::Stdin => todo!(),
        Stream::Stdout => command.execute(Stdio::inherit(), file, io::stderr()),
        Stream::Stderr => command.execute(Stdio::inherit(), io::stdout(), file),
    }
}

fn append(src: Box<Expr>, stream: Stream, dest: String) {
    let Expr::Command { name, args } = *src else {
        panic!("expected command before pipe");
    };

    let file = OpenOptions::new().append(true).open(dest).unwrap();

    let command = Command::new(name, args);

    match stream {
        Stream::Stdin => todo!(),
        Stream::Stdout => command.execute(Stdio::inherit(), file, io::stderr()),
        Stream::Stderr => command.execute(Stdio::inherit(), io::stdout(), file),
    }
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

    let Some(src_path) = find_path(&src_name) else {
        eprintln!("{}: command not found", src_name);
        return;
    };

    let mut cmd1 = std::process::Command::new(&src_path)
        .arg0(src_path.file_name().unwrap())
        .args(src_args)
        .stdout(pipe_writer)
        .spawn()
        .unwrap();

    let Some(dest_path) = find_path(&dest_name) else {
        eprintln!("{}: command not found", dest_name);
        return;
    };

    let mut cmd2 = std::process::Command::new(&dest_path)
        .arg0(dest_path.file_name().unwrap())
        .args(dest_args)
        .stdin(pipe_reader)
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    let command_output = cmd2;

    cmd1.wait().unwrap();
}
