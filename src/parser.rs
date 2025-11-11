use std::iter::Peekable;

pub struct Parser;

impl Parser {
    pub fn parse(cmd: &str) -> Vec<String> {
        let mut chars = cmd.chars().peekable();
        let mut args = Vec::new();

        while let Some(char) = chars.peek() {
            if char == &' ' {
                Self::whitespace(&mut chars);
                continue;
            }

            let arg = match char {
                '\'' => Self::single_quote_argument(&mut chars),
                '"' => Self::double_quote_argument(&mut chars),
                _c => Self::normal_argument(&mut chars),
            };

            args.push(arg);
        }

        args
    }

    fn normal_argument<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> String {
        let mut word = String::new();

        while let Some(char) = chars.peek() {
            match char {
                &'\'' => word += &Self::single_quote_argument(chars),
                &'\"' => word += &Self::double_quote_argument(chars),
                &'\\' => word.push(Self::escape_char(chars)),
                &' ' => break,
                _c => word.push(chars.next().unwrap()),
            }
        }

        word
    }

    fn escape_char<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> char {
        // consume escape char
        chars.next().unwrap();
        chars
            .next()
            .expect("expected escaped character after backslash")
    }

    fn single_quote_argument<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> String {
        let mut word = String::new();

        // consume opening quote
        chars.next().unwrap();

        while let Some(char) = chars.peek() {
            match char {
                &'\'' => break,
                _c => word.push(chars.next().unwrap()),
            }
        }

        // consume closing quote
        chars.next().expect("expected closing single quote");

        if let Some('"') = chars.peek() {
            word += &Self::double_quote_argument(chars);
        }

        if let Some('\'') = chars.peek() {
            word += &Self::single_quote_argument(chars);
        }

        word
    }

    fn double_quote_argument<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> String {
        let mut word = String::new();

        // consume opening quote
        chars.next().unwrap();

        while let Some(char) = chars.peek() {
            match char {
                &'"' => break,
                _c => word.push(chars.next().unwrap()),
            }
        }

        // consume closing quote
        chars.next().expect("expected closing double quote");

        if let Some('"') = chars.peek() {
            word += &Self::double_quote_argument(chars);
        }

        if let Some('\'') = chars.peek() {
            word += &Self::single_quote_argument(chars);
        }

        word
    }

    fn whitespace<I: Iterator<Item = char>>(chars: &mut Peekable<I>) {
        while let Some(&' ') = chars.peek() {
            chars.next().unwrap();
        }
    }
}

