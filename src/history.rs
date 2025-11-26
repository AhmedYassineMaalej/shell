pub struct History {
    commands: Vec<String>,
    cursor: Option<usize>,
}

impl History {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            cursor: None,
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

    pub fn add(&mut self, command: String) {
        self.commands.push(command);
        self.cursor = None;
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

impl<'a> IntoIterator for &'a History {
    type Item = &'a String;

    type IntoIter = <&'a Vec<String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.iter()
    }
}

