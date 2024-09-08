use std::io::{self, BufRead};

mod token;
use token::Token;

mod error;
use error::JSONError;

pub struct JSONParser<R: BufRead> {
    reader: R,
    tokens: Vec<Token>,
    current_line: i64,
}

impl<R: BufRead> JSONParser<R> {
    fn lex(&mut self) -> Result<(), io::Error> {
        loop {
            let mut buf = Vec::<u8>::new();
            match self.reader.read_until(b'\n', &mut buf) {
                Ok(0) => {
                    return Ok(());
                }
                Ok(_) => {
                    let s = String::from_utf8(buf).expect("from_utf8 failed");
                    for c in s.chars() {
                        match c {
                            '{' => {
                                self.tokens.push(Token::OpenBrace);
                            }
                            '}' => {
                                self.tokens.push(Token::ClosedBrace);
                            }
                            '\n' => {
                                self.tokens.push(Token::NewLine);
                            }
                            _ => self.tokens.push(Token::GenericChar(c)),
                        }
                    }

                    buf = s.into_bytes();
                    buf.clear();
                }
                Err(err) => {
                    return Err(err);
                }
            }
            self.current_line += 1;
        }
    }

    fn new(reader: R) -> JSONParser<R> {
        JSONParser {
            reader,
            tokens: Vec::new(),
            current_line: 1,
        }
    }

    fn build_json_err(&self, message: String) -> JSONError {
        JSONError::new(message.clone(), self.current_line)
    }

    pub fn check_valid(reader: R) -> Result<(), JSONError> {
        let p = &mut Self::new(reader);
        p.lex().unwrap();

        p.current_line = 1;
        let mut is_inside_object = false;
        let mut is_json_ended = false;
        for token in &p.tokens {
            match token {
                Token::OpenBrace => {
                    is_inside_object = true;
                }
                Token::ClosedBrace => {
                    if !is_inside_object {
                        return Err(p.build_json_err(format!("Unexpected {}", token)));
                    }
                    is_inside_object = false;
                    is_json_ended = true;
                }
                Token::NewLine => {
                    //ignore
                }
                Token::GenericChar(_) => {
                    return Err(p.build_json_err(format!("Unexpected {}", token)))
                }
            }
        }
        if !is_json_ended {
            return Err(p.build_json_err(String::from("Unexpected EOF")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn should_lex_with_open_brace() {
        let mut jp = JSONParser {
            reader: "{".as_bytes(),
            tokens: Vec::new(),
            current_line: 1,
        };
        jp.lex();
        assert_eq!(jp.tokens, Vec::from([Token::OpenBrace,]));
    }

    #[test]
    fn should_lex_with_open_closed_brace() {
        let mut jp = JSONParser {
            reader: "{}".as_bytes(),
            tokens: Vec::new(),
            current_line: 1,
        };
        jp.lex();
        assert_eq!(
            jp.tokens,
            Vec::from([Token::OpenBrace, Token::ClosedBrace,])
        );
    }

    #[test]
    fn should_lex_emoji() {
        let mut jp = JSONParser {
            reader: "😊".as_bytes(),
            tokens: Vec::new(),
            current_line: 1,
        };
        jp.lex();
        assert_eq!(jp.tokens, Vec::from([Token::GenericChar('😊'),]));
    }

    #[test]
    fn should_lex_normal_text() {
        let mut jp = JSONParser {
            reader: "hello".as_bytes(),
            tokens: Vec::new(),
            current_line: 1,
        };
        jp.lex().unwrap();
        assert_eq!(
            jp.tokens,
            Vec::from([
                Token::GenericChar('h'),
                Token::GenericChar('e'),
                Token::GenericChar('l'),
                Token::GenericChar('l'),
                Token::GenericChar('o'),
            ])
        );
    }
}

#[cfg(test)]
mod check_valid_tests {
    use crate::parser::JSONParser;

    #[test]
    fn should_not_report_error_for_obj() {
        let res = JSONParser::check_valid("{}".as_bytes());
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn should_report_error_for_not_closed_brace() {
        let found_err = JSONParser::check_valid("{".as_bytes()).unwrap_err();
        assert_eq!("Unexpected EOF: at line 1", found_err.to_string())
    }

    #[test]
    fn should_report_error_for_closed_brace_outside_obj() {
        let found_err = JSONParser::check_valid("}".as_bytes()).unwrap_err();
        assert_eq!("Unexpected '}': at line 1", found_err.to_string())
    }

    #[test]
    fn should_report_error_for_random_char() {
        let found_err = JSONParser::check_valid("{a".as_bytes()).unwrap_err();
        assert_eq!("Unexpected 'a': at line 1", found_err.to_string())
    }

    #[test]
    fn should_report_unexpected_eof_for_empty_file() {
        let found_err = JSONParser::check_valid("".as_bytes()).unwrap_err();
        assert_eq!("Unexpected EOF: at line 1", found_err.to_string())
    }

    #[test]
    fn should_not_report_error_for_new_line_at_the_end_of_file() {
        let res = JSONParser::check_valid("{}\n".as_bytes());
        assert_eq!(Ok(()), res)
    }
}
