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

// command1 | command2 | command3
// after parsing:
// Pipe {
//   Pipe {
//      command1,
//      command2,
//   },
//   command3,
// executing:
// create pipe 1->2
// create pipe 2->3
// wait for command3
// wait for command2
// wait for command1
// Pipe1.execute(stdin, stdout) {
//      create pipe 1->2;
//      Pipe2.execute(pipe1->2.writer, )
// }

impl Executable for Expr {
    fn execute<I, O, E>(&self, stdin: I, stdout: O, stderr: E) -> Option<std::process::Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + io::Write,
        E: Into<Stdio> + io::Write,
    {
        match self {
            Expr::Command { name, args } => {
                let command = Command::new(name.clone(), args.clone());
                command.execute(stdin, stdout, stderr)
            }
            Expr::Redirect { src, stream, dest } => {
                let file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(dest)
                    .unwrap();

                match stream {
                    Stream::Stdin => todo!(),
                    Stream::Stdout => src.execute(stdin, file, stderr),
                    Stream::Stderr => src.execute(stdin, stdout, file),
                }
            }
            Expr::Append { src, stream, dest } => {
                let file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(dest)
                    .unwrap();

                match stream {
                    Stream::Stdin => todo!(),
                    Stream::Stdout => src.execute(stdin, file, stderr),
                    Stream::Stderr => src.execute(stdin, stdout, file),
                }
            }
            Expr::Pipe { src, dest } => {
                let (pipe_reader, pipe_writer) = std::io::pipe().unwrap();

                let child1 = src.execute(stdin, pipe_writer, stderr);
                let child2 = dest.execute(pipe_reader, stdout, io::stderr());
                child2.map(|mut child| child.wait());

                child1
            }
        }
    }
}

fn command(name: String, args: Vec<String>) {
    let command = Command::new(name, args);
    if let Some(mut child) = command.execute(Stdio::inherit(), io::stdout(), io::stderr()) {
        child.wait();
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
        Stream::Stdout => command
            .execute(Stdio::inherit(), file, io::stderr())
            .map(|mut child| child.wait()),
        Stream::Stderr => command
            .execute(Stdio::inherit(), io::stdout(), file)
            .map(|mut child| child.wait()),
    };
}

fn pipe(src: Box<Expr>, dest: Box<Expr>) {
    let Expr::Command {
        name: src_name,
        args: src_args,
    } = *src
    else {
        panic!("expected command before pipe");
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
