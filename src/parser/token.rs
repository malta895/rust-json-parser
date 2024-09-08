use core::fmt;

#[derive(PartialEq, Debug)]
pub enum Token {
    OpenBrace,
    ClosedBrace,
    NewLine,
    GenericChar(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token_str: String = match self {
            Token::OpenBrace => String::from("'{'"),
            Token::ClosedBrace => String::from("'}'"),
            Token::NewLine => String::from('\n'),
            Token::GenericChar(c) => format!("'{}'", c),
        };
        write!(f, "{}", token_str)
    }
}