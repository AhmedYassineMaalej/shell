use std::{
    env::{self, split_paths},
    io::Write,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::{self, exit, Stdio},
};

pub enum Command {
    Builtin(BuiltinCommand),
    Binary(String),
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        if let Ok(builtin) = BuiltinCommand::try_from(value) {
            Self::Builtin(builtin)
        } else {
            Self::Binary(String::from(value))
        }
    }
}

pub struct CommandContext {
    command: Command,
    args: Vec<String>,
}

pub struct CommandOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub success: bool,
}

impl CommandOutput {
    pub fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
            success: true,
        }
    }
}

impl CommandContext {
    pub fn new(command: Command, args: Vec<String>) -> Self {
        Self { command, args }
    }

    pub fn execute(self) -> CommandOutput {
        match self.command {
            Command::Builtin(builtin) => builtin.execute(self.args),
            Command::Binary(path) => Self::execute_binary(path, self.args),
        }
    }

    fn execute_binary(path: String, args: Vec<String>) -> CommandOutput {
        let mut output = CommandOutput::new();

        let Some(path) = find_path(&path) else {
            writeln!(&mut output.stderr, "{}: command not found", path);
            output.success = false;
            return output;
        };

        let mut command = process::Command::new(&path);
        command.arg0(path.file_name().unwrap());
        command.args(args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let command_output = command.spawn().unwrap().wait_with_output().unwrap();

        output.success = command_output.status.success();
        output.stdout = command_output.stdout;
        output.stderr = command_output.stderr;

        output
    }
}

pub enum BuiltinCommand {
    Echo,
    Cd,
    Pwd,
    Type,
    Exit,
}

impl BuiltinCommand {
    pub fn execute(self, args: Vec<String>) -> CommandOutput {
        match self {
            BuiltinCommand::Echo => Self::echo(args),
            BuiltinCommand::Cd => Self::cd(args),
            BuiltinCommand::Pwd => Self::pwd(args),
            BuiltinCommand::Type => Self::type_(args),
            BuiltinCommand::Exit => Self::exit(args),
        }
    }

    fn echo(args: Vec<String>) -> CommandOutput {
        let mut output = CommandOutput::new();
        writeln!(&mut output.stdout, "{}", args.join(" "));
        output
    }

    fn cd(args: Vec<String>) -> CommandOutput {
        let mut output = CommandOutput::new();
        let path = if args.len() >= 1 {
            args.into_iter().next().unwrap()
        } else {
            String::from("~")
        };

        if path == "~" {
            let home_dir = env::home_dir().unwrap();
            env::set_current_dir(&home_dir).unwrap();
            return output;
        }

        let current_directory = env::current_dir().unwrap();

        match current_directory.join(&path).canonicalize() {
            Ok(new_dir) => std::env::set_current_dir(&new_dir).unwrap(),
            Err(_e) => {
                writeln!(output.stderr, "cd: {}: No such file or directory", path);
            }
        }

        output
    }

    fn pwd(args: Vec<String>) -> CommandOutput {
        let mut output = CommandOutput::new();

        if args.len() > 0 {
            writeln!(output.stderr, "pwd: too many arguments");
            return output;
        }

        let current_directory = env::current_dir().unwrap();
        writeln!(output.stdout, "{}", current_directory.display());
        output
    }

    fn type_(args: Vec<String>) -> CommandOutput {
        let mut output = CommandOutput::new();
        let cmd = args.first().unwrap().as_str();

        if BuiltinCommand::try_from(cmd).is_ok() {
            writeln!(output.stdout, "{cmd} is a shell builtin");
            return output;
        }

        match find_path(cmd) {
            Some(file) => writeln!(output.stdout, "{cmd} is {}", file.display()),
            None => writeln!(output.stderr, "{cmd}: not found"),
        };

        output
    }

    fn exit(args: Vec<String>) -> ! {
        exit(0);
    }
}

impl TryFrom<&str> for BuiltinCommand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "echo" => Ok(Self::Echo),
            "pwd" => Ok(Self::Pwd),
            "cd" => Ok(Self::Cd),
            "type" => Ok(Self::Type),
            "exit" => Ok(Self::Exit),
            s => Err(format!("unknown builtin: {s}")),
        }
    }
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
