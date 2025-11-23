use std::{
    fs::OpenOptions,
    io::{Write, pipe},
    process::{Command, Stdio},
};

#[test]
fn test_piping() {
    let (pipe_reader, pipe_writer) = pipe().unwrap();

    let mut cmd1 = Command::new("tail")
        .arg("-f")
        .arg("test.txt")
        .stdout(pipe_writer)
        .spawn()
        .unwrap();

    let mut file = OpenOptions::new().append(true).open("test.txt").unwrap();
    file.write_all("fourth line\nfifth line\n".as_bytes());

    let cmd2 = Command::new("head")
        .arg("-n 5")
        .stdin(pipe_reader)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    cmd1.wait().unwrap();

    println!("{}", String::from_utf8(cmd2.stdout).unwrap())
}
