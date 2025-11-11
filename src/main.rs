#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::Command,
};

mod parser;
use parser::Parser;

fn main() {
    let stdin = io::stdin();
    let mut buf = String::new();
    let mut current_directory = std::env::current_dir().unwrap();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();

        let args: Vec<String> = Parser::parse(buf.trim());
        let command = args[0].as_str();

        match command {
            "exit" => {
                let _code: usize = args[1].parse().unwrap();
                break;
            }
            "echo" => {
                let message = args[1..].join(" ");
                println!("{message}");
            }
            "pwd" => {
                println!("{}", current_directory.display())
            }
            "cd" => {
                change_directory(&mut current_directory, &args[1]);
            }
            "type" => type_command(&args[1]),
            cmd => run_command(cmd, &args[1..]),
        }

        buf.clear();
    }
}

fn parse_args(mut cmd: &str) -> Vec<String> {
    let mut args = Vec::new();

    while let Some((rest, arg)) = parse_argument(cmd) {
        cmd = rest;
        args.push(arg);
    }

    args
}

fn parse_argument(mut cmd: &str) -> Option<(&str, String)> {
    cmd = cmd.trim_start();

    if let Some(rest) = cmd.strip_prefix('\'') {
        let end = rest.find('\'').unwrap();

        if end == 0 {
            return parse_argument(&rest[1..]);
        }

        return Some((&rest[end + 1..], String::from(&rest[..end])));
    }

    if let Some(rest) = cmd.strip_prefix('\"') {
        let end = rest.find('\"').unwrap();

        if end == 0 {
            return parse_argument(&rest[1..]);
        }

        return Some((&rest[end + 1..], String::from(&rest[..end])));
    }

    if let Some(idx) = cmd.find(' ') {
        return Some((&cmd[idx + 1..], String::from(&cmd[..idx]).replace("''", "")));
    }

    if !cmd.is_empty() {
        return Some(("", String::from(cmd).replace("''", "")));
    }

    None
}

fn change_directory(current_directory: &mut PathBuf, path: &str) {
    if path == "~" {
        let home_dir = std::env::home_dir().unwrap();
        *current_directory = home_dir;
        std::env::set_current_dir(&current_directory).unwrap();
        return;
    }

    match current_directory.join(path).canonicalize() {
        Ok(new_dir) => {
            *current_directory = new_dir;
            std::env::set_current_dir(&current_directory).unwrap();
        }
        Err(_e) => {
            println!("cd: {}: No such file or directory", path);
        }
    }
}

fn run_command(cmd: &str, args: &[String]) {
    let Some(executable) = find_executable(cmd) else {
        println!("{cmd}: not found");
        return;
    };

    Command::new(executable)
        .arg0(cmd)
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn find_executable(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var("PATH").unwrap();

    for dir in path.split(':') {
        let mut dir = PathBuf::from(dir);
        dir.push(cmd);

        if dir.exists() {
            let metadata = std::fs::metadata(&dir).unwrap();
            let permissions = metadata.permissions();

            if permissions.mode() & 0o111 != 0 {
                return Some(dir);
            }
        }
    }

    None
}

fn type_command(cmd: &str) {
    if ["exit", "type", "echo", "pwd"].contains(&cmd) {
        println!("{cmd} is a shell builtin");
        return;
    }

    match find_executable(cmd) {
        Some(file) => println!("{cmd} is {}", file.display()),
        None => println!("{cmd}: not found"),
    }
}
