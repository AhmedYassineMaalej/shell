pub struct Tokenizer {
    chars: Vec<char>,
    position: usize,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub enum Token {
    Ampersand,
    Greater,
    Literal(String),
    OneGreater,
    Pipe,
    TwoGreater,
    ZeroGreater,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            position: 0,
            tokens: Vec::new(),
        }
    }

    fn peek(&self) -> Option<&char> {
        self.chars.get(self.position)
    }

    fn next(&mut self) -> Option<&char> {
        let c = self.chars.get(self.position);
        self.position += 1;
        c
    }

    pub fn parse(&mut self) {
        while let Some(char) = self.peek() {
            match char {
                ' ' => self.whitespace(),
                '>' => {
                    self.next().unwrap();
                    self.tokens.push(Token::Greater);
                }
                '0' if self.chars.get(self.position + 1) == Some(&'>') => {
                    self.next().unwrap();
                    self.next().unwrap();
                    self.tokens.push(Token::ZeroGreater);
                }
                '1' if self.chars.get(self.position + 1) == Some(&'>') => {
                    self.next().unwrap();
                    self.next().unwrap();
                    self.tokens.push(Token::OneGreater);
                }
                '2' if self.chars.get(self.position + 1) == Some(&'>') => {
                    self.next().unwrap();
                    self.next().unwrap();
                    self.tokens.push(Token::TwoGreater);
                }
                '|' => {
                    self.next().unwrap();
                    self.tokens.push(Token::Pipe);
                }
                '&' => {
                    self.next().unwrap();
                    self.tokens.push(Token::Ampersand);
                }
                _ => self.literal(),
            }
        }
    }

    fn literal(&mut self) {
        let mut literal = String::new();

        while let Some(char) = self.peek() {
            match char {
                &' ' | &'>' | &'&' | '|' => break,
                &'\'' => literal.push_str(&self.single_quote_literal()),
                &'\"' => literal.push_str(&self.double_quote_literal()),
                &'\\' => {
                    // consume backslash
                    self.next().unwrap();
                    literal.push(
                        *self
                            .next()
                            .expect("expected escaped character after backslash"),
                    )
                }
                _ => literal.push(*self.next().unwrap()),
            }
        }

        self.tokens.push(Token::Literal(literal))
    }

    fn single_quote_literal(&mut self) -> String {
        let mut literal = String::new();

        // consume opening quote
        self.next();

        while let Some(char) = self.next() {
            match char {
                &'\'' => break,
                c => literal.push(*c),
            }
        }

        literal
    }

    fn double_quote_literal(&mut self) -> String {
        let mut literal = String::new();

        // consume opening quote
        self.next();

        while let Some(char) = self.next() {
            match char {
                &'"' => break,
                &'\\' => match self.next() {
                    Some('\\') => literal.push('\\'),
                    Some('\"') => literal.push('\"'),
                    Some(c) => {
                        literal.push('\\');
                        literal.push(*c);
                    }
                    None => panic!("expected character after backslash"),
                },
                c => literal.push(*c),
            }
        }

        literal
    }

    fn whitespace(&mut self) {
        while let Some(&' ') = self.peek() {
            self.next().unwrap();
        }
    }

    fn done(&self) -> bool {
        self.position == self.chars.len()
    }

    pub fn tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokenizer = Self::new(input);
        tokenizer.parse();
        tokenizer.tokens()
    }
}
