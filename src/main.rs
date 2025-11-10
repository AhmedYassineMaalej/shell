#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    stdin.read_line(&mut buf).unwrap();

    match buf.trim() {
        s => println!("{s}: command not found"),
    }
}
