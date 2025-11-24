use std::{
    collections::HashSet,
    env::{self, split_paths},
    fs,
    io::Write,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{self, Child, Stdio, exit},
};

pub struct Cd {
    target_directory: Option<PathBuf>,
}

impl Executable for Cd {
    fn execute<I, O, E>(&self, _stdin: I, mut stdout: O, _stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        let path = match &self.target_directory {
            Some(path) => path.clone(),
            None => PathBuf::from("~"),
        };

        if path == Path::new("~") {
            let home_dir = env::home_dir().unwrap();
            env::set_current_dir(&home_dir).unwrap();
            return None;
        }

        let current_directory = env::current_dir().unwrap();

        match current_directory.join(&path).canonicalize() {
            Ok(new_dir) => std::env::set_current_dir(&new_dir).unwrap(),
            Err(_e) => {
                writeln!(stdout, "cd: {}: No such file or directory", path.display()).unwrap();
            }
        }

        None
    }
}

pub struct Type {
    command: String,
}

impl Executable for Type {
    fn execute<I, O, E>(&self, _stdin: I, mut stdout: O, mut stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        if BUILTINS.contains(&self.command.as_str()) {
            writeln!(stdout, "{} is a shell builtin", self.command).unwrap();
            return None;
        }

        match find_path(&self.command) {
            Some(file) => writeln!(stdout, "{} is {}", self.command, file.display()).unwrap(),
            None => writeln!(stderr, "{}: not found", self.command).unwrap(),
        }

        None
    }
}

pub struct Echo {
    args: Vec<String>,
}

impl Executable for Echo {
    fn execute<I, O, E>(&self, _stdin: I, mut stdout: O, _stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        writeln!(stdout, "{}", self.args.join(" ")).unwrap();
        None
    }
}

pub struct Pwd;

impl Executable for Pwd {
    fn execute<I, O, E>(&self, _stdin: I, mut stdout: O, _stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        // TODO:
        // if !args.is_empty() {
        //     writeln!(output.stderr, "pwd: too many arguments");
        //     return output;
        // }

        let current_directory = env::current_dir().unwrap();
        writeln!(stdout, "{}", current_directory.display()).unwrap();
        None
    }
}

pub struct Exit {
    code: Option<i32>,
}

pub struct Binary {
    path: String,
    args: Vec<String>,
}

impl Executable for Binary {
    fn execute<I, O, E>(&self, stdin: I, stdout: O, mut stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        let Some(path) = find_path(&self.path) else {
            writeln!(stderr, "{}: command not found", self.path).unwrap();
            return None;
        };

        let mut command = process::Command::new(&path);
        command.arg0(path.file_name().unwrap());
        command.args(&self.args);
        command.stdin(stdin);
        command.stdout(stdout);
        command.stderr(stderr);

        Some(command.spawn().unwrap())
    }
}

impl Executable for Exit {
    fn execute<I, O, E>(&self, _stdin: I, _stdout: O, _stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        exit(self.code.unwrap_or_default());
    }
}

const BUILTINS: [&str; 5] = ["echo", "cd", "pwd", "type", "exit"];

pub fn get_commands() -> HashSet<String> {
    let mut commands: HashSet<String> = HashSet::new();

    // add builtin commands
    for builtin in BUILTINS {
        commands.insert(builtin.to_string());
    }

    // add binaries
    let path = env::var("PATH").unwrap();

    for dir in split_paths(&path) {
        let dir = fs::read_dir(dir).unwrap();

        for binary in dir {
            let command = binary.unwrap().file_name().into_string().unwrap();
            commands.insert(command);
        }
    }

    commands
}

pub fn find_path(command_name: &str) -> Option<PathBuf> {
    let path = env::var("PATH").unwrap();

    for mut dir in split_paths(&path) {
        dir.push(command_name);

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

pub enum Command {
    Cd(Cd),
    Pwd(Pwd),
    Type(Type),
    Echo(Echo),
    Exit(Exit),
    Binary(Binary),
}

impl Executable for Command {
    fn execute<I, O, E>(&self, stdin: I, stdout: O, stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        match self {
            Command::Cd(cd) => cd.execute(stdin, stdout, stderr),
            Command::Pwd(pwd) => pwd.execute(stdin, stdout, stderr),
            Command::Type(type_) => type_.execute(stdin, stdout, stderr),
            Command::Echo(echo) => echo.execute(stdin, stdout, stderr),
            Command::Exit(exit) => exit.execute(stdin, stdout, stderr),
            Command::Binary(binary) => binary.execute(stdin, stdout, stderr),
        }
    }
}

impl Command {
    pub fn new(name: String, args: Vec<String>) -> Self {
        match name.as_str() {
            "cd" => Self::Cd(Cd {
                target_directory: args.into_iter().next().map(PathBuf::from),
            }),

            "pwd" => Self::Pwd(Pwd),
            "echo" => Self::Echo(Echo { args }),
            "exit" => Self::Exit(Exit { code: None }),
            "type" => Self::Type(Type {
                command: args.into_iter().next().unwrap(),
            }),
            _ => Self::Binary(Binary { path: name, args }),
        }
    }
}

pub trait Executable {
    fn execute<I, O, E>(&self, stdin: I, stdout: O, stderr: E) -> Option<Child>
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write;
}
