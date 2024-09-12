use core::fmt;

#[derive(PartialEq, Debug)]
pub enum Token {
    OpenBrace,
    ClosedBrace,
    OpenBracket,
    ClosedBracket,
    NewLine,
    DoubleQuotes,
    Column,
    Comma,
    StringLiteral(String),
    Number(f64),
    BoolTrue,
    BoolFalse,
    Null
}

const OPEN_BRACE: &str= "{";
const CLOSED_BRACE: &str = "}";
const NEW_LINE: &str = "\n"; //TODO: make sure this works on windows too
const DOUBLE_QUOTES: &str = "\"";
const COLUMN: &str = ":";
const COMMA: &str = ",";
const OPEN_BRACKET: &str = "[";
const CLOSED_BRACKET: &str = "]";


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let token_str: String = match self {
            Token::OpenBrace => String::from(OPEN_BRACE),
            Token::ClosedBrace => String::from(CLOSED_BRACE),
            Token::NewLine => String::from(NEW_LINE),
            Token::DoubleQuotes => String::from(DOUBLE_QUOTES),
            Token::Column => String::from(COLUMN),
            Token::Comma => String::from(COMMA),
            Token::Number(_) => String::from("<number>"),
            Token::BoolTrue | Token::BoolFalse => String::from("<boolean>"),
            Token::Null => String::from("<null>"),
            Token::StringLiteral(_) => String::from("<string literal>"),
            Token::OpenBracket => String::from(OPEN_BRACKET),
            Token::ClosedBracket => String::from(CLOSED_BRACKET),
            
        };
        write!(f, "'{}'", token_str)
    }
}