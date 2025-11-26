pub struct History {
    commands: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add(&mut self, command: String) {
        self.commands.push(command);
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
