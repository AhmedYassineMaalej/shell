#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut buf = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();

        let mut words = buf.trim().split(' ');
        let command = words.next().unwrap();

        match command {
            "exit" => {
                let code: usize = words.next().unwrap().parse().unwrap();
                break;
            }
            cmd => println!("{cmd}: command not found"),
        }

        buf.clear();
    }
}
