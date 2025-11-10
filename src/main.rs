#[allow(unused_imports)]
use std::io::{self, Write};

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
            "type" => {
                let cmd = words[1];
                if ["exit", "type", "echo"].contains(&cmd) {
                    println!("{cmd} is a shell builtin");
                } else {
                    println!("{cmd}: not found")
                }
            }
            cmd => println!("{cmd}: command not found"),
        }

        buf.clear();
    }
}
