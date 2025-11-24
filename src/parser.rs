use crate::tokenizer::Token;

#[derive(Debug)]
pub enum Expr {
    Command {
        name: String,
        args: Vec<String>,
    },
    Redirect {
        src: Box<Expr>,
        stream: Stream,
        dest: String,
    },
    Append {
        src: Box<Expr>,
        stream: Stream,
        dest: String,
    },
    Pipe {
        src: Box<Expr>,
        dest: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Stream {
    Stdin,
    Stdout,
    Stderr,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    ast: Option<Expr>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            ast: None,
        }
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.position).cloned();
        self.position += 1;
        t
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.position).cloned()
    }

    pub fn parse(&mut self) {
        self.ast = Some(self.command());

        while let Some(token) = self.peek() {
            match token {
                Token::Ampersand => todo!(),
                Token::Greater | Token::OneGreater | Token::TwoGreater => {
                    self.ast = Some(self.redirect());
                }
                Token::Literal(_) => todo!(),
                Token::Pipe => self.ast = Some(self.pipe()),
                Token::ZeroGreater => todo!(),
                Token::TwoDoubleGreater | Token::OneDoubleGreater | Token::DoubleGreater => {
                    self.ast = Some(self.append());
                }
                Token::ZeroDoubleGreater => {
                    todo!()
                }
            }
        }
    }

    fn command(&mut self) -> Expr {
        let Token::Literal(name) = self.next().expect("no command name found") else {
            panic!("expected command name")
        };

        let mut args = Vec::new();

        while let Some(Token::Literal(s)) = self.peek() {
            args.push(s);
            self.next().unwrap();
        }

        Expr::Command { name, args }
    }

    fn redirect(&mut self) -> Expr {
        let src = Box::new(self.ast.take().unwrap());

        let stream = match self.next().unwrap() {
            Token::Greater | Token::OneGreater => Stream::Stdout,
            Token::ZeroGreater => Stream::Stdin,
            Token::TwoGreater => Stream::Stderr,
            t => panic!("expected redirect, found {t:?}"),
        };

        let Some(Token::Literal(dest)) = self.next() else {
            panic!("expected destination after redirect");
        };

        Expr::Redirect { src, stream, dest }
    }

    fn pipe(&mut self) -> Expr {
        let src = Box::new(self.ast.take().unwrap());
        self.next(); // consume pipe

        let Expr::Command { name, args } = self.command() else {
            panic!("expected command after pipe");
        };

        let dest = Box::new(Expr::Command { name, args });
        Expr::Pipe { src, dest }
    }

    pub fn ast(self) -> Expr {
        self.ast.unwrap()
    }

    fn append(&mut self) -> Expr {
        let src = Box::new(self.ast.take().unwrap());

        let stream = match self.next().unwrap() {
            Token::DoubleGreater | Token::OneDoubleGreater => Stream::Stdout,
            Token::ZeroDoubleGreater => Stream::Stdin,
            Token::TwoDoubleGreater => Stream::Stderr,
            t => panic!("expected append, found {t:?}"),
        };

        let Some(Token::Literal(dest)) = self.next() else {
            panic!("expected destination after append");
        };

        Expr::Append { src, stream, dest }
    }
}
