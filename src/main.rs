#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut buf = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();

        match buf.trim() {
            s => println!("{s}: command not found"),
        }

        buf.clear();
    }
}
