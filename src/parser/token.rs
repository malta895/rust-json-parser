use core::fmt;

#[derive(PartialEq, Debug)]
pub enum Token {
    OpenBrace,
    ClosedBrace,
    NewLine,
    DoubleQuotes,
    Column,
    Comma,
    StringLiteral(String),
    Number,
    BoolTrue,
    BoolFalse
}

const OPEN_BRACE: &str= "{";
const CLOSED_BRACE: &str = "}";
const NEW_LINE: &str = "\n"; //TODO: make sure this works on windows too
const DOUBLE_QUOTES: &str = "\"";
const COLUMN: &str = ":";
const COMMA: &str = ",";


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token_str: String = match self {
            Token::OpenBrace => String::from(OPEN_BRACE),
            Token::ClosedBrace => String::from(CLOSED_BRACE),
            Token::NewLine => String::from(NEW_LINE),
            Token::DoubleQuotes => String::from(DOUBLE_QUOTES),
            Token::Column => String::from(COLUMN),
            Token::Comma => String::from(COMMA),
            Token::Number => String::from("<number>"),
            Token::BoolTrue | Token::BoolFalse => String::from("<boolean>"),
            Token::StringLiteral(_) => String::from("<string literal>"),
        };
        write!(f, "'{}'", token_str)
    }
}