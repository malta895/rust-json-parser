use std::io::BufRead;

use super::{error::JSONError, token::Token};

#[derive(PartialEq, Clone)]
enum State {
    Normal,

    ObjValue,
    ValueNumber,
    ValueTrue(char),
    ValueFalse(char),
    ValueNull(char),

    ArrValue,

    ValueStringLiteral,
    Escaping,
}

pub fn lex<R: BufRead>(mut reader: R) -> Result<Vec<Token>, JSONError> {
    //TODO: this is not a lexer anymore, as it was not necessary. Remove the parser and do everything here
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
                        ('{', State::Normal | State::ObjValue) => {
                            tokens.push(Token::OpenBrace);
                            state = State::Normal;
                        }

                        ('}', State::Normal | State::ObjValue) => {
                            tokens.push(Token::ClosedBrace);
                        }
                        ('}', State::ValueNumber) => {
                            tokens.push(Token::Number);
                            tokens.push(Token::ClosedBrace);
                        }

                        ('[', State::ObjValue) => {
                            tokens.push(Token::OpenBracket);
                            state = State::ArrValue;
                        }
                        (']', State::ArrValue | State::Normal) => {
                            tokens.push(Token::ClosedBracket);
                            state = State::Normal;
                        }

                        ('\n', State::Normal) => {
                            tokens.push(Token::NewLine);
                        }
                        ('\n', State::ValueNumber) => {
                            state = State::Normal;
                            tokens.push(Token::Number);
                            tokens.push(Token::NewLine);
                        }

                        (':', State::Normal) => {
                            tokens.push(Token::Column);
                            state = State::ObjValue;
                        }

                        (',', State::Normal) => {
                            tokens.push(Token::Comma);
                        }
                        (',', State::ValueNumber) => {
                            tokens.push(Token::Number);
                            tokens.push(Token::Comma);
                            state = State::Normal;
                        }

                        (' ', State::Normal | State::ObjValue) => {
                            // ignore space
                        }

                        ('0'..='9', State::ObjValue | State::ValueNumber) => {
                            state = State::ValueNumber
                        }

                        ('t', State::ObjValue) => state = State::ValueTrue('t'),
                        ('r', State::ValueTrue('t')) => state = State::ValueTrue('r'),
                        ('u', State::ValueTrue('r')) => state = State::ValueTrue('u'),
                        ('e', State::ValueTrue('u')) => {
                            tokens.push(Token::BoolTrue);
                            state = State::Normal;
                        }

                        ('f', State::ObjValue) => state = State::ValueFalse('f'),
                        ('a', State::ValueFalse('f')) => state = State::ValueFalse('a'),
                        ('l', State::ValueFalse('a')) => state = State::ValueFalse('l'),
                        ('s', State::ValueFalse('l')) => state = State::ValueFalse('s'),
                        ('e', State::ValueFalse('s')) => {
                            tokens.push(Token::BoolFalse);
                            state = State::Normal;
                        }

                        ('n', State::ObjValue) => state = State::ValueNull('n'),
                        ('u', State::ValueNull('n')) => state = State::ValueNull('u'),
                        ('l', State::ValueNull('u')) => state = State::ValueNull('l'),
                        ('l', State::ValueNull('l')) => {
                            tokens.push(Token::Null);
                            state = State::Normal;
                        }

                        ('"' | '\\', State::Escaping) => {
                            curr_string_literal.push(c);
                            state = State::ValueStringLiteral;
                        }
                        ('\\', State::ValueStringLiteral) => {
                            state = State::Escaping;
                        }
                        ('"', State::Normal | State::ObjValue | State::ArrValue) => {
                            state = State::ValueStringLiteral;
                            tokens.push(Token::DoubleQuotes);
                        }
                        ('"', State::ValueStringLiteral) => {
                            tokens.push(Token::StringLiteral(curr_string_literal.clone()));
                            curr_string_literal.clear();
                            state = State::Normal;
                            tokens.push(Token::DoubleQuotes);
                        }

                        (_, State::ValueStringLiteral) => curr_string_literal.push(c),
                        (_, _) => return Err(JSONError::new(format!("Unexpected '{}'", c), 1)),
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

    fn run_expected_error_test_case_with(input_str: &str, expected_error: JSONError) {
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
        run_test_case_with(
            "{\"😊\":\"\"}",
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
            ]),
        );
    }

    #[test]
    fn should_report_err_lex_normal_text() {
        run_expected_error_test_case_with("hello", JSONError::new(format!("Unexpected 'h'"), 1));
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
    fn should_include_escape_char_when_itself_escaped() {
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
    fn shuold_not_allow_escaping_unallowed_chars() {
        run_expected_error_test_case_with(
            "{\"ab\\c\":\"\"",
            JSONError::new(format!("Unexpected 'c'"), 1),
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
    fn should_ignore_spaces_outside_string_literals() {
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
    fn should_consider_spaces_in_string_literals() {
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

    #[test]
    fn should_lex_a_number() {
        run_test_case_with(
            "{ \"key\": 123456789}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::Number,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_a_number_before_new_line() {
        run_test_case_with(
            "{ \"key\": 123456789\n}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::Number,
                Token::NewLine,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_a_number_before_comma() {
        run_test_case_with(
            "{ \"key\": 1234567890, \"key2\":\"\"}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::Number,
                Token::Comma,
                Token::DoubleQuotes,
                Token::StringLiteral("key2".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("".to_string()),
                Token::DoubleQuotes,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_true() {
        run_test_case_with(
            "{ \"key\": true}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::BoolTrue,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_true_before_new_line() {
        run_test_case_with(
            "{ \"key\": true\n}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::BoolTrue,
                Token::NewLine,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_true_before_comma() {
        run_test_case_with(
            "{ \"key\": true, \"key2\":\"\"}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::BoolTrue,
                Token::Comma,
                Token::DoubleQuotes,
                Token::StringLiteral("key2".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("".to_string()),
                Token::DoubleQuotes,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_false() {
        run_test_case_with(
            "{ \"key\": false}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::BoolFalse,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_not_lex_misspelled_false() {
        run_expected_error_test_case_with(
            "{ \"key\": fsale}",
            JSONError::new(format!("Unexpected 's'"), 1),
        )
    }

    #[test]
    fn should_lex_false_before_comma() {
        run_test_case_with(
            "{ \"key\": false, \"key2\":\"\"}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::BoolFalse,
                Token::Comma,
                Token::DoubleQuotes,
                Token::StringLiteral("key2".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("".to_string()),
                Token::DoubleQuotes,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_not_lex_misspelled_true() {
        run_expected_error_test_case_with(
            "{ \"key\": ture}",
            JSONError::new(format!("Unexpected 'u'"), 1),
        )
    }

    #[test]
    fn should_lex_null() {
        run_test_case_with(
            "{ \"key\": null}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::Null,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_empty_obj_val() {
        run_test_case_with(
            "{ \"key\": {}}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::OpenBrace,
                Token::ClosedBrace,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_empty_array_val() {
        run_test_case_with(
            "{ \"key\": []}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::OpenBracket,
                Token::ClosedBracket,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_array_with_inner_value() {
        run_test_case_with(
            "{ \"key\": [\"val\"]}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::OpenBracket,
                Token::DoubleQuotes,
                Token::StringLiteral("val".to_string()),
                Token::DoubleQuotes,
                Token::ClosedBracket,
                Token::ClosedBrace,
            ]),
        )
    }


    #[test]
    fn should_lex_obj_with_inner_value() {
        run_test_case_with(
            "{ \"key\": {\"inner_key\":\"inner_val\"}}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("inner_key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("inner_val".to_string()),
                Token::DoubleQuotes,                
                Token::ClosedBrace,
                Token::ClosedBrace,
            ]),
        )
    }

    #[test]
    fn should_lex_obj_with_inner_value_new_line() {
        run_test_case_with(
            "{ \"key\": {\n\"inner_key\":\"inner_val\"\n}\n}",
            Vec::from([
                Token::OpenBrace,
                Token::DoubleQuotes,
                Token::StringLiteral("key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::OpenBrace,
                Token::NewLine,
                Token::DoubleQuotes,
                Token::StringLiteral("inner_key".to_string()),
                Token::DoubleQuotes,
                Token::Column,
                Token::DoubleQuotes,
                Token::StringLiteral("inner_val".to_string()),
                Token::DoubleQuotes,                
                Token::NewLine,
                Token::ClosedBrace,
                Token::NewLine,
                Token::ClosedBrace,
            ]),
        )
    }
}
