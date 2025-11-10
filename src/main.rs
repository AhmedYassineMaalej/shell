#[allow(unused_imports)]
use std::io::{self, Write};
use std::{os::unix::fs::PermissionsExt, path::PathBuf};

fn main() {
    let stdin = io::stdin();
    let mut buf = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();

        let mut words: Vec<&str> = buf.split_whitespace().collect();
        let command = words[0];

        match command {
            "exit" => {
                let _code: usize = words[1].parse().unwrap();
                break;
            }
            "echo" => {
                let message = words[1..].join(" ");
                println!("{message}");
            }
            "type" => type_command(words[1]),
            cmd => println!("{cmd}: command not found"),
        }

        buf.clear();
    }
}

fn type_command(cmd: &str) {
    if ["exit", "type", "echo"].contains(&cmd) {
        println!("{cmd} is a shell builtin");
        return;
    }

    let path = std::env::var("PATH").unwrap();

    for dir in path.split(':') {
        let mut dir = PathBuf::from(dir);
        dir.push(cmd);

        if dir.exists() {
            let metadata = std::fs::metadata(&dir).unwrap();
            let permissions = metadata.permissions();

            if permissions.mode() & 0o111 != 0 {
                println!("{cmd} is {}", dir.display());
                return;
            }
        }
    }

    println!("{cmd}: not found")
}
