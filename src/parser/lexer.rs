use std::io::{BufRead};

use super::{error::JSONError, token::Token};

#[derive(PartialEq, Clone)]
enum State {
    Normal,
    StringLiteral,
    Escaping,
}

pub fn lex<R: BufRead>(mut reader: R) -> Result<Vec<Token>, JSONError> {
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
                let mut curr_string_literal = String::new();
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
                        (' ', State::Normal) => {
                            // ignore space
                        }
                        (_, State::Escaping) => {
                            // TODO: we should probably only allow to escape " and \
                            curr_string_literal.push(c);
                            state = State::StringLiteral;
                        }
                        ('\\', State::StringLiteral) => {
                            state = State::Escaping;
                        }
                        ('"', _) => {
                            if state == State::Normal {
                                state = State::StringLiteral;
                            } else {
                                tokens.push(
                                    Token::StringLiteral(curr_string_literal.clone())
                                );
                                curr_string_literal.clear();
                                state = State::Normal;
                            }
                            tokens.push(Token::DoubleQuotes);
                        }
                        (_, State::StringLiteral) => curr_string_literal.push(c),
                        (_, _) => {
                            return Err(
                                JSONError::new(
                                    format!("Unexpected '{}'", c),
                                    1
                                )
                            )
                        },
                    }
                }

                buf = s.into_bytes();
                buf.clear();
            }
            Err(err) => {
                // TODO: implement line count
                return Err(JSONError::new(err.to_string(), 1));
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

    fn run_expected_error_test_case_with(input_str: &str, expected_error: JSONError){
        let reader = input_str.as_bytes();
        let found_tokens = lex(reader).unwrap_err();
        assert_eq!(found_tokens, expected_error);
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
        run_test_case_with("{\"😊\":\"\"}", 
        Vec::from([
            Token::OpenBrace,
            Token::DoubleQuotes,
            Token::StringLiteral("😊".to_string()),
            Token::DoubleQuotes,
            Token::Column,
            Token::DoubleQuotes,
            Token::StringLiteral("".to_string()),
            Token::DoubleQuotes,
            Token::ClosedBrace,
        ]));
    }

    #[test]
    fn should_report_err_lex_normal_text() {
        run_expected_error_test_case_with(
            "hello",
            JSONError::new(
                format!("Unexpected 'h'"),
                 1
            )
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
                Token::StringLiteral("{:".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("".to_string()),
                Token::DoubleQuotes,
            ]),
        )
    }

    #[test]
    fn should_ignore_escaped_double_quotes() {
        run_test_case_with(
            "{\"ab\\\"c\":\"\"",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("ab\"c".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("".to_string()),
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
                Token::StringLiteral("ab\\c".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("".to_string()),
                Token::DoubleQuotes,
            ]),
        )
    }

    #[test]
    fn should_lex_comma_correctly() {
        run_test_case_with(
            "{\"key\":\"val\",\"key2\":\"val\"}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("val".to_string()),
                Token::DoubleQuotes,
                Token::Comma,
                Token::DoubleQuotes,
                Token::StringLiteral("key2".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("val".to_string()),
                Token::DoubleQuotes,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_ignore_spaces_outside_string_literals(){
        run_test_case_with(
            "{  \"key\":\"val\",\n  \"key2\":\"val\"}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("val".to_string()),
                Token::DoubleQuotes,
                Token::Comma,
                Token::NewLine,
                Token::DoubleQuotes,
                Token::StringLiteral("key2".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("val".to_string()),
                Token::DoubleQuotes,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_consider_spaces_in_string_literals(){
        run_test_case_with(
            "{  \"key\":\"va l\",\n  \"ke y2\":\"val\"}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("va l".to_string()),
                Token::DoubleQuotes,
                Token::Comma,
                Token::NewLine,
                Token::DoubleQuotes,
                Token::StringLiteral("ke y2".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("val".to_string()),
                Token::DoubleQuotes,
                Token::ClosedBrace,
            ]),
        )
    }
}
