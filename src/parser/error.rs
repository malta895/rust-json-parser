use core::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JSONError {
    message: String,
    line: i64,
}

impl fmt::Display for JSONError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: at line {}", self.message, self.line)
    }
}

impl JSONError {
    pub fn new(message: String, line: i64) -> JSONError {
        JSONError { message, line }
    }
}
