use std::{
    fs::OpenOptions,
    io::{Write, pipe},
    process::{Command, Stdio},
};

#[test]
fn test_piping() {
    let (pipe_reader, pipe_writer) = pipe().unwrap();

    let mut binding = Command::new("tail");
    let mut cmd1 = binding.arg("-f").arg("test.txt").stdout(pipe_writer);
    cmd1.spawn().unwrap();

    let cmd2 = Command::new("head")
        .arg("-n 5")
        .stdin(pipe_reader)
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    println!("{}", String::from_utf8(cmd2.stdout).unwrap())
}
