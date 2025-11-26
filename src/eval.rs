use std::{
    fs::OpenOptions,
    io::{self},
    process::Stdio,
};

use crate::{
    commands::{Command, Executable},
    parser::{Expr, Stream},
    shell::Shell,
};

impl Executable for Expr {
    fn execute<I, O, E>(
        &self,
        shell: &Shell,
        stdin: I,
        stdout: O,
        stderr: E,
    ) -> Option<std::process::Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + io::Write,
        E: Into<Stdio> + io::Write,
    {
        match self {
            Expr::Command { name, args } => {
                let command = Command::new(name.clone(), args.clone());
                command.execute(shell, stdin, stdout, stderr)
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
                    Stream::Stdout => src.execute(shell, stdin, file, stderr),
                    Stream::Stderr => src.execute(shell, stdin, stdout, file),
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
                    Stream::Stdout => src.execute(shell, stdin, file, stderr),
                    Stream::Stderr => src.execute(shell, stdin, stdout, file),
                }
            }
            Expr::Pipe { src, dest } => {
                let (pipe_reader, pipe_writer) = std::io::pipe().unwrap();

                let child1 = src.execute(shell, stdin, pipe_writer, stderr);
                let child2 = dest.execute(shell, pipe_reader, stdout, io::stderr());
                child2.map(|mut child| child.wait());

                child1
            }
        }
    }
}
