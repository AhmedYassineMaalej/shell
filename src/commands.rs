use std::{
    env::{self, split_paths},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{self, exit, Stdio},
};

pub enum Command {
    Builtin(BuiltinCommand),
    Binary(PathBuf),
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        if let Ok(builtin) = BuiltinCommand::try_from(value) {
            Self::Builtin(builtin)
        } else {
            Self::Binary(PathBuf::from(value))
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
        let mut output = CommandOutput::new();
        match self.command {
            Command::Builtin(builtin) => builtin.execute(&mut output.stdout, self.args),
            Command::Binary(path) => {
                let mut command = process::Command::new(path);
                command.args(self.args);
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());

                let command_output = command.spawn().unwrap().wait_with_output().unwrap();

                output.success = command_output.status.success();
                output.stdout = command_output.stdout;
                output.stderr = command_output.stderr;
            }
        }

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
    pub fn execute<W: Write>(self, writer: &mut W, args: Vec<String>) {
        match self {
            BuiltinCommand::Echo => Self::echo(writer, args),
            BuiltinCommand::Cd => Self::cd(writer, args),
            BuiltinCommand::Pwd => Self::pwd(writer, args),
            BuiltinCommand::Type => Self::type_(writer, args),
            BuiltinCommand::Exit => Self::exit(writer, args),
        }
    }

    fn echo<W: Write>(writer: &mut W, args: Vec<String>) {
        writeln!(writer, "{}", args.join(" "));
    }

    fn cd<W: Write>(writer: &mut W, args: Vec<String>) {
        let path = if args.len() >= 1 {
            args.into_iter().next().unwrap()
        } else {
            String::from("~")
        };

        if path == "~" {
            let home_dir = env::home_dir().unwrap();
            env::set_current_dir(&home_dir).unwrap();
            return;
        }

        let current_directory = env::current_dir().unwrap();

        match current_directory.join(&path).canonicalize() {
            Ok(new_dir) => std::env::set_current_dir(&new_dir).unwrap(),
            Err(_e) => {
                writeln!(writer, "cd: {}: No such file or directory", path);
            }
        }
    }

    fn pwd<W: Write>(writer: &mut W, args: Vec<String>) {
        if args.len() > 0 {
            writeln!(writer, "pwd: too many arguments");
            return;
        }

        let current_directory = env::current_dir().unwrap();
        writeln!(writer, "{}", current_directory.display());
    }

    fn type_<W: Write>(writer: &mut W, args: Vec<String>) {
        let cmd = args.first().unwrap().as_str();

        if BuiltinCommand::try_from(cmd).is_ok() {
            writeln!(writer, "{cmd} is a shell builtin");
            return;
        }

        match find_path(cmd) {
            Some(file) => writeln!(writer, "{cmd} is {}", file.display()),
            None => writeln!(writer, "{cmd}: not found"),
        };
    }

    fn exit<W: Write>(writer: &mut W, args: Vec<String>) {
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
