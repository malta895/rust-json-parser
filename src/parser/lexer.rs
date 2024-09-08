use std::io::{self, BufRead};

use super::token::Token;

enum State {
    Normal,
    StringLiteral,
}

struct Lexer {
    state: State,
}

pub fn lex<R: BufRead>(mut reader: R) -> Result<Vec<Token>, io::Error> {
    let mut tokens = Vec::new();
    loop {
        let mut buf = Vec::<u8>::new();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => {
                return Ok(tokens);
            }
            Ok(_) => {
                let s = String::from_utf8(buf).expect("from_utf8 failed");
                for c in s.chars() {
                    match c {
                        '{' => {
                            tokens.push(Token::OpenBrace);
                        }
                        '}' => {
                            tokens.push(Token::ClosedBrace);
                        }
                        '"' => {
                            tokens.push(Token::DoubleQuotes);
                        }
                        '\n' => {
                            tokens.push(Token::NewLine);
                        }
                        ':' => {
                            tokens.push(Token::Column);
                        }
                        _ => tokens.push(Token::GenericChar(c)),
                    }
                }

                buf = s.into_bytes();
                buf.clear();
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}


#[cfg(test)]
mod lexer_tests {
    use crate::parser::JSONParser;

    use super::*;

    fn run_test_case_with(input_str: &str, expected_tokens: Vec<Token>) {
        let reader = input_str.as_bytes();
        let found_tokens = lex(reader).unwrap();
        assert_eq!(found_tokens, expected_tokens);
    }

    #[test]
    fn should_lex_with_open_brace() {
        run_test_case_with("{", Vec::from([Token::OpenBrace]));
    }

    #[test]
    fn should_lex_with_open_closed_brace() {
        run_test_case_with("{}", Vec::from([Token::OpenBrace, Token::ClosedBrace]));
    }

    #[test]
    fn should_lex_emoji() {
        run_test_case_with("😊", Vec::from([Token::GenericChar('😊')]));
    }

    #[test]
    fn should_lex_normal_text() {
        run_test_case_with(
            "hello",
            Vec::from([
                Token::GenericChar('h'),
                Token::GenericChar('e'),
                Token::GenericChar('l'),
                Token::GenericChar('l'),
                Token::GenericChar('o'),
            ]),
        );
    }

    #[test]
    fn should_lex_double_qoutes() {
        run_test_case_with(
            "\"",
            Vec::from([
                Token::DoubleQuotes,
            ]),
        );
    }

    #[test]
    fn should_lex_column() {
        run_test_case_with(
            ":",
            Vec::from([
                Token::Column,
            ]),
        );
    }
}
