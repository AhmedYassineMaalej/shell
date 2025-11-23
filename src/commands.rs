use std::{
    collections::HashSet,
    env::{self, split_paths},
    fs,
    io::{Read, Write, pipe},
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::{self, Stdio, exit},
};

pub struct Cd {
    target_directory: Option<PathBuf>,
}

impl Executable for Cd {
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        let path = match &self.target_directory {
            Some(path) => path.clone(),
            None => PathBuf::from("~"),
        };

        if path == PathBuf::from("~") {
            let home_dir = env::home_dir().unwrap();
            env::set_current_dir(&home_dir).unwrap();
            return;
        }

        let current_directory = env::current_dir().unwrap();

        match current_directory.join(&path).canonicalize() {
            Ok(new_dir) => std::env::set_current_dir(&new_dir).unwrap(),
            Err(_e) => {
                writeln!(stdout, "cd: {}: No such file or directory", path.display());
            }
        }
    }
}

pub struct Type {
    command: String,
}

impl Executable for Type {
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        if BUILTINS.contains(&self.command.as_str()) {
            writeln!(stdout, "{} is a shell builtin", self.command);
            return;
        }

        match find_path(&self.command) {
            Some(file) => writeln!(stdout, "{} is {}", self.command, file.display()),
            None => writeln!(stderr, "{}: not found", self.command),
        };
    }
}

pub struct Echo {
    args: Vec<String>,
}

impl Executable for Echo {
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        writeln!(stdout, "{}", self.args.join(" "));
    }
}

pub struct Pwd;

impl Executable for Pwd {
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
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
        writeln!(stdout, "{}", current_directory.display());
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
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        let Some(path) = find_path(&self.path) else {
            writeln!(stderr, "{}: command not found", self.path);
            return;
        };

        let mut command = process::Command::new(&path);
        command.arg0(path.file_name().unwrap());
        command.args(&self.args);
        command.stdin(stdin);
        command.stdout(stdout);
        command.stderr(stderr);

        command.spawn().unwrap();
    }
}

impl Executable for Exit {
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
        exit(self.code.unwrap_or_default());
    }
}

// impl From<&str> for Command {
//     fn from(value: &str) -> Self {
//         if let Ok(builtin) = BuiltinCommand::try_from(value) {
//             Self::Builtin(builtin)
//         } else {
//             Self::Binary(String::from(value))
//         }
//     }
// }

// pub struct CommandContext {
//     command: Command,
//     args: Vec<String>,
// }

// pub struct CommandOutput {
//     pub stdout: Vec<u8>,
//     pub stderr: Vec<u8>,
//     pub success: bool,
// }
//
// impl CommandOutput {
//     pub fn new() -> Self {
//         Self {
//             stdout: Vec::new(),
//             stderr: Vec::new(),
//             success: true,
//         }
//     }
// }
//
// impl CommandContext {
//     pub fn new(command: Command, args: Vec<String>) -> Self {
//         Self { command, args }
//     }
//
//     pub fn execute(self) -> CommandOutput {
//         match self.command {
//             Command::Builtin(builtin) => builtin.execute(self.args),
//             Command::Binary(path) => Self::execute_binary(path, self.args),
//         }
//     }
//
//     fn execute_binary(path: String, args: Vec<String>) -> CommandOutput {
//         let mut output = CommandOutput::new();
//
//         let Some(path) = find_path(&path) else {
//             writeln!(&mut output.stderr, "{}: command not found", path);
//             output.success = false;
//             return output;
//         };
//
//         let mut command = process::Command::new(&path);
//         command.arg0(path.file_name().unwrap());
//         command.args(args);
//         command.stdout(Stdio::piped());
//         command.stderr(Stdio::piped());
//
//         let command_output = command.spawn().unwrap().wait_with_output().unwrap();
//
//         output.success = command_output.status.success();
//         output.stdout = command_output.stdout;
//         output.stderr = command_output.stderr;
//
//         output
//     }
//
//     pub fn execute_binary_piped(
//         src_path: String,
//         src_args: Vec<String>,
//         dest_path: String,
//         dest_args: Vec<String>,
//     ) -> CommandOutput {
//         let mut output = CommandOutput::new();
//
//         let (pipe_reader, pipe_writer) = pipe().unwrap();
//
//         let Some(src_path) = find_path(&src_path) else {
//             writeln!(&mut output.stderr, "{}: command not found", src_path);
//             output.success = false;
//             return output;
//         };
//
//         let mut cmd1 = process::Command::new(&src_path)
//             .arg0(src_path.file_name().unwrap())
//             .args(src_args)
//             .stdout(pipe_writer)
//             .spawn()
//             .unwrap();
//
//         let Some(dest_path) = find_path(&dest_path) else {
//             writeln!(&mut output.stderr, "{}: command not found", dest_path);
//             output.success = false;
//             return output;
//         };
//
//         let mut cmd2 = process::Command::new(&dest_path)
//             .arg0(dest_path.file_name().unwrap())
//             .args(dest_args)
//             .stdin(pipe_reader)
//             .stdout(Stdio::inherit())
//             .spawn()
//             .unwrap()
//             .wait_with_output()
//             .unwrap();
//
//         let command_output = cmd2;
//
//         cmd1.wait().unwrap();
//
//         output.success = command_output.status.success();
//         output.stdout = command_output.stdout;
//         output.stderr = command_output.stderr;
//
//         output
//     }
// }

const BUILTINS: [&'static str; 5] = ["echo", "cd", "pwd", "type", "exit"];

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
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
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
            "cd" => {
                return Self::Cd(Cd {
                    target_directory: args.into_iter().next().map(PathBuf::from),
                });
            }

            "pwd" => return Self::Pwd(Pwd),
            "echo" => return Self::Echo(Echo { args }),
            "exit" => return Self::Exit(Exit { code: None }),
            "type" => {
                return Self::Type(Type {
                    command: args.into_iter().next().unwrap(),
                });
            }
            _ => Self::Binary(Binary { path: name, args }),
        }
    }
}

pub trait Executable {
    fn execute<I, O, E>(&self, mut stdin: I, mut stdout: O, mut stderr: E)
    where
        I: Into<Stdio>,
        O: Into<Stdio> + Write,
        E: Into<Stdio> + Write,
    {
    }
}
