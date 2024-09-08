use core::fmt;

#[derive(PartialEq, Debug)]
pub enum Token {
    OpenBrace,
    ClosedBrace,
    NewLine,
    DoubleQuotes,
    // Column,
    GenericChar(char),
}

pub const OPEN_BRACE: &str= "{";
pub const CLOSED_BRACE: &str = "}";
pub const NEW_LINE: &str = "\n"; //TODO: make sure this works on windows too
pub const DOUBLE_QUOTES: &str = "\"";
pub const COLUMN: &str = ":";


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token_str: String = match self {
            Token::OpenBrace => String::from(OPEN_BRACE),
            Token::ClosedBrace => String::from(CLOSED_BRACE),
            Token::NewLine => String::from(NEW_LINE),
            Token::DoubleQuotes => String::from(DOUBLE_QUOTES),
            Token::GenericChar(c) => format!("{}", c),
        };
        write!(f, "'{}'", token_str)
    }
}