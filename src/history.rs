use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

#[derive(Default)]
pub struct History {
    commands: Vec<String>,
    cursor: Option<usize>,
    append_start: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            cursor: None,
            append_start: 0,
        }
    }

    pub fn prev(&mut self) -> Option<&String> {
        if self.commands.is_empty() {
            return None;
        }

        self.cursor = match self.cursor {
            Some(i) => Some(i.saturating_sub(1)),
            None => Some(self.commands.len() - 1),
        };

        self.commands.get(self.cursor.unwrap())
    }

    pub fn next(&mut self) -> Option<&String> {
        if self.commands.is_empty() {
            return None;
        }

        match self.cursor {
            None => None,
            Some(i) => {
                if i < self.commands.len() - 1 {
                    self.cursor = Some(i + 1);
                }
                self.commands.get(self.cursor.unwrap())
            }
        }
    }

    pub fn add(&mut self, command: String) {
        self.commands.push(command);
        self.cursor = None;
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn read_from_file(&mut self, file: PathBuf) {
        let content = fs::read_to_string(file).unwrap();
        for line in content.lines() {
            self.add(line.to_string());
        }
    }

    pub fn write_to_file(&self, file: PathBuf) {
        fs::write(file, self.commands.join("\n") + "\n").unwrap();
    }

    pub fn append_to_file(&mut self, file: PathBuf) {
        let mut file = OpenOptions::new().append(true).open(file).unwrap();

        write!(
            file,
            "{}",
            self.commands[self.append_start..]
                .iter()
                .fold(String::new(), |acc, x| acc + x + "\n")
        )
        .unwrap();

        self.append_start = self.commands.len();
    }
}

impl<'a> IntoIterator for &'a History {
    type Item = &'a String;

    type IntoIter = <&'a Vec<String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.iter()
    }
}
