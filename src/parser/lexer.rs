use std::io::{self, BufRead};

use super::token::Token;

#[derive(PartialEq, Clone)]
enum State {
    Normal,
    StringLiteral,
    Escaping,
}

pub fn lex<R: BufRead>(mut reader: R) -> Result<Vec<Token>, io::Error> {
    let mut tokens = Vec::new();
    let mut state = State::Normal;
    loop {
        let mut buf = Vec::<u8>::new();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => {
                return Ok(tokens);
            }
            Ok(_) => {
                let s = String::from_utf8(buf).expect("from_utf8 failed");
                for c in s.chars() {
                    let current_state = state.clone();
                    match (c, current_state) {
                        ('{', State::Normal) => {
                            tokens.push(Token::OpenBrace);
                        }
                        ('}', State::Normal) => {
                            tokens.push(Token::ClosedBrace);
                        }
                        ('\n', State::Normal) => {
                            tokens.push(Token::NewLine);
                        }
                        (':', State::Normal) => {
                            tokens.push(Token::Column);
                        }
                        (',', State::Normal) => {
                            tokens.push(Token::Comma);
                        }
                        (_, State::Escaping) => {
                            // TODO: we should probably only allow to escape " and \
                            tokens.push(Token::GenericChar(c));
                            state = State::StringLiteral;
                        }
                        ('\\', State::StringLiteral) => {
                            state = State::Escaping;
                        }
                        ('"', _) => {
                            tokens.push(Token::DoubleQuotes);

                            if state == State::Normal {
                                state = State::StringLiteral;
                            } else {
                                state = State::Normal;
                            }
                        }

                        (_, _) => tokens.push(Token::GenericChar(c)),
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
        run_test_case_with("\"", Vec::from([Token::DoubleQuotes]));
    }

    #[test]
    fn should_lex_column() {
        run_test_case_with(":", Vec::from([Token::Column]));
    }

    #[test]
    fn should_ignore_tokens_when_in_string_literal() {
        run_test_case_with(
            "{\"{:\":\"\"",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::GenericChar('{'),
                Token::GenericChar(':'),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::DoubleQuotes,
            ]),
        )
    }

    #[test]
    fn shuold_ignore_escaped_double_quotes() {
        run_test_case_with(
            "{\"ab\\\"c\":\"\"",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::GenericChar('a'),
                Token::GenericChar('b'),
                Token::GenericChar('"'),
                Token::GenericChar('c'),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::DoubleQuotes,
            ]),
        )
    }

    #[test]
    fn shuold_include_escape_char_when_itself_escaped() {
        run_test_case_with(
            "{\"ab\\\\c\":\"\"",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::GenericChar('a'),
                Token::GenericChar('b'),
                Token::GenericChar('\\'),
                Token::GenericChar('c'),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::DoubleQuotes,
            ]),
        )
    }

    #[test]
    fn shuold_lex_comma_correctly() {
        run_test_case_with(
            "{\"key\":\"val\",\"key2\":\"val\"}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::GenericChar('k'),
                Token::GenericChar('e'),
                Token::GenericChar('y'),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::GenericChar('v'),
                Token::GenericChar('a'),
                Token::GenericChar('l'),
                Token::DoubleQuotes,
                Token::Comma,
                Token::DoubleQuotes,
                Token::GenericChar('k'),
                Token::GenericChar('e'),
                Token::GenericChar('y'),
                Token::GenericChar('2'),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::GenericChar('v'),
                Token::GenericChar('a'),
                Token::GenericChar('l'),
                Token::DoubleQuotes,
                Token::ClosedBrace,
            ]),
        )
    }
}
