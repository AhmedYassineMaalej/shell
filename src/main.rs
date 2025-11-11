#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::Command,
};

fn main() {
    let stdin = io::stdin();
    let mut buf = String::new();
    let mut current_directory = std::env::current_dir().unwrap();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();

        let args: Vec<&str> = buf.split_whitespace().collect();
        let command = args[0];

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
                change_directory(&mut current_directory, args[1]);
            }
            "type" => type_command(args[1]),
            cmd => run_command(cmd, &args[1..]),
        }

        buf.clear();
    }
}

fn change_directory(current_directory: &mut PathBuf, mut path: &str) {
    if path.starts_with('/') {
        let dir = PathBuf::from(path);
        if dir.exists() {
            std::env::set_current_dir(&dir).unwrap();
            *current_directory = dir;
        } else {
            println!("cd: {}: No such file or directory", dir.display())
        }
        return;
    }

    match current_directory.join(path).canonicalize() {
        Ok(new_dir) => {
            *current_directory = new_dir;
            std::env::set_current_dir(&current_directory).unwrap();
        }
        Err(e) => {
            println!("cd: {}: No such file or directory", path);
        }
    }

    if path == "./" || path == "." {
        return;
    }

    if path == "../" || path == ".." {
        current_directory.pop();
        return;
    }

    if path.starts_with("./") {
        path = path.strip_prefix("./").unwrap();
    }

    let new_dir = current_directory.join(PathBuf::from(path));
    if new_dir.exists() {
        *current_directory = new_dir;
    } else {
        println!("cd: {}: No such file or directory", path);
    }
}

fn run_command(cmd: &str, args: &[&str]) {
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
