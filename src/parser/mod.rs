use std::io::{self, BufRead};

mod token;
use lexer::lex;
use token::Token;

mod error;
use error::JSONError;

mod lexer;

pub struct JSONParser<R: BufRead> {
    reader: R,
    tokens: Vec<Token>,
    current_line: i64,
}

impl<R: BufRead> JSONParser<R> {
    fn lex(&mut self) -> Result<(), JSONError> {
        let tokens = lex(&mut self.reader);
        match tokens {
            Ok(tokens) => {
                self.tokens = tokens;
                Ok(())
            }
            Err(lex_error) => Err(lex_error),
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
        p.lex()?;

        p.current_line = 1;
        let mut is_inside_object = false;
        let mut is_json_ended = false;
        let mut is_inside_literal = false;
        let mut is_after_comma = false;
        for token in &p.tokens {
            match token {
                Token::OpenBrace => {
                    is_after_comma = false;
                    is_inside_object = true;
                }
                Token::ClosedBrace => {
                    if !is_inside_object {
                        return Err(p.build_json_err(format!("Unexpected {}", token)));
                    }
                    if is_after_comma {
                        return Err(p.build_json_err(format!("Unexpected {}", token)));
                    }
                    is_inside_object = false;
                    is_json_ended = true;
                }
                Token::NewLine => {
                    // ignore for now
                }
                Token::DoubleQuotes => {
                    is_after_comma = false;
                    is_inside_literal = !is_inside_literal;
                }
                Token::Column => {
                    // ignore for now
                }
                Token::Comma => {
                    is_after_comma = true;
                }
                Token::Space => {
                    // ignore for now
                    // TODO: do we need it?
                }
                Token::StringLiteral(_) => todo!(),
                Token::GenericChar(_) => {
                    if !is_inside_literal {
                        return Err(p.build_json_err(format!("Unexpected {}", token)));
                    }
                }
            }
        }
        if !is_json_ended {
            return Err(p.build_json_err(String::from("Unexpected EOF")));
        }
        Ok(())
    }
}

mod lexer_tests {
    use super::*;

    fn run_test_case_with(input_str: &str, expected_tokens: Vec<Token>) {
        let mut jp = JSONParser {
            reader: input_str.as_bytes(),
            tokens: Vec::new(),
            current_line: 1,
        };
        jp.lex();

        assert_eq!(jp.tokens, expected_tokens);
    }

    #[test]
    fn should_lex_with_open_brace() {
        run_test_case_with("{", Vec::from([Token::OpenBrace]));
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

    #[test]
    fn should_recognize_base_case() {
        let res = JSONParser::check_valid("{\"\":\"\"}\n".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_base_case_with_key_val() {
        let res = JSONParser::check_valid("{\"key\":\"value\"}\n".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_string_with_spaces() {
        let res = JSONParser::check_valid("{  \"key\":\"va l\",\n  \"ke y2\":\"val\"}".as_bytes());
        assert_eq!(Ok(()), res)
    }
}
