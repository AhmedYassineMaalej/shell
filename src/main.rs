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
            cmd => println!("{cmd}: command not found"),
        }

        buf.clear();
    }
}
