use std::{io::BufRead, iter::StepBy};

use super::{error::JSONError, token::Token};

#[derive(PartialEq, Clone, Copy)]
enum NumberState {
    Sign,
    Exp,
    Point,
    ExpSign,

    LeadingZero,
    Integer,
    Decimal,
    ExpLeadingZero,
    ExpInteger,
}

impl NumberState {
    pub fn is_final(self) -> bool {
        match self {
            Self::LeadingZero
            | Self::Integer
            | Self::Decimal
            | Self::ExpLeadingZero
            | Self::ExpInteger => true,
            _ => false,
        }
    }

    pub fn is_exp(self) -> bool {
        self == Self::ExpLeadingZero || self == Self::ExpInteger
    }
}

#[derive(PartialEq, Clone)]
enum State {
    Normal,

    ValueNumber(NumberState),

    ValueTrue(char),
    ValueFalse(char),
    ValueNull(char),

    ValueStringLiteral,
    Escaping,
}

pub fn lex<R: BufRead>(mut reader: R) -> Result<Vec<Token>, JSONError> {
    let mut tokens = Vec::new();

    loop {
        let mut buf = Vec::<u8>::new();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => {
                return Ok(tokens);
            }
            Ok(_) => {
                let s = String::from_utf8(buf).expect("from_utf8 failed");
                let mut curr_string_literal = String::new();
                let mut state = State::Normal;
                for c in s.chars() {
                    state = match (c, &state) {
                        ('\\', State::ValueStringLiteral) => State::Escaping,
                        ('"', State::ValueStringLiteral) => {
                            tokens.push(Token::StringLiteral(curr_string_literal.clone()));
                            curr_string_literal.clear();
                            tokens.push(Token::DoubleQuotes);
                            State::Normal
                        }
                        (_, State::ValueStringLiteral) => {
                            curr_string_literal.push(c);
                            State::ValueStringLiteral
                        }
                        ('"' | '\\', State::Escaping) => {
                            curr_string_literal.push(c);
                            State::ValueStringLiteral
                        }

                        ('"', State::Normal) => {
                            tokens.push(Token::DoubleQuotes);
                            State::ValueStringLiteral
                        }

                        ('{', State::Normal) => {
                            tokens.push(Token::OpenBrace);
                            State::Normal
                        }

                        ('}', State::Normal) => {
                            tokens.push(Token::ClosedBrace);
                            State::Normal
                        }
                        ('}', State::ValueNumber(n)) if n.is_final() => {
                            tokens.push(Token::Number);
                            tokens.push(Token::ClosedBrace);
                            State::Normal
                        }

                        ('[', State::Normal) => {
                            tokens.push(Token::OpenBracket);
                            state
                        }
                        (']', State::Normal) => {
                            tokens.push(Token::ClosedBracket);
                            State::Normal
                        }
                        (']', State::ValueNumber(n)) if n.is_final() => {
                            tokens.push(Token::Number);
                            tokens.push(Token::ClosedBracket);
                            State::Normal
                        }

                        ('\n', State::Normal) => {
                            tokens.push(Token::NewLine);
                            State::Normal
                        }
                        ('\n', State::ValueNumber(n)) if n.is_final() => {
                            tokens.push(Token::Number);
                            tokens.push(Token::NewLine);
                            State::Normal
                        }

                        (':', State::Normal) => {
                            tokens.push(Token::Column);
                            State::Normal
                        }

                        (',', State::Normal) => {
                            tokens.push(Token::Comma);
                            State::Normal
                        }
                        (',', State::ValueNumber(n)) if n.is_final() => {
                            tokens.push(Token::Number);
                            tokens.push(Token::Comma);
                            State::Normal
                        }

                        (' ', State::Normal) => State::Normal,

                        ('-' | '+', State::Normal) => State::ValueNumber(NumberState::Sign),
                        ('-' | '+', State::ValueNumber(NumberState::Exp)) => {
                            State::ValueNumber(NumberState::ExpSign)
                        }
                        ('e' | 'E', State::ValueNumber(n)) if n.is_final() && !n.is_exp() => {
                            State::ValueNumber(NumberState::Exp)
                        }
                        ('0', State::ValueNumber(NumberState::Exp)) => {
                            State::ValueNumber(NumberState::ExpLeadingZero)
                        }
                        ('0', State::Normal | State::ValueNumber(NumberState::Sign)) => {
                            State::ValueNumber(NumberState::LeadingZero)
                        }
                        (
                            '1'..='9',
                            State::ValueNumber(NumberState::Exp | NumberState::ExpSign),
                        ) => State::ValueNumber(NumberState::ExpInteger),
                        ('1'..='9', State::Normal | State::ValueNumber(NumberState::Sign)) => {
                            State::ValueNumber(NumberState::Integer)
                        }
                        ('0'..='9', State::ValueNumber(NumberState::Point)) => {
                            State::ValueNumber(NumberState::Decimal)
                        }
                        ('0'..='9', State::ValueNumber(n_type))
                            if *n_type != NumberState::LeadingZero =>
                        {
                            State::ValueNumber(*n_type)
                        }
                        (
                            '.',
                            State::ValueNumber(NumberState::Integer | NumberState::LeadingZero),
                        ) => State::ValueNumber(NumberState::Point),

                        ('t', State::Normal) => State::ValueTrue('t'),
                        ('r', State::ValueTrue('t')) => State::ValueTrue('r'),
                        ('u', State::ValueTrue('r')) => State::ValueTrue('u'),
                        ('e', State::ValueTrue('u')) => {
                            tokens.push(Token::BoolTrue);
                            State::Normal
                        }

                        ('f', State::Normal) => State::ValueFalse('f'),
                        ('a', State::ValueFalse('f')) => State::ValueFalse('a'),
                        ('l', State::ValueFalse('a')) => State::ValueFalse('l'),
                        ('s', State::ValueFalse('l')) => State::ValueFalse('s'),
                        ('e', State::ValueFalse('s')) => {
                            tokens.push(Token::BoolFalse);
                            State::Normal
                        }

                        ('n', State::Normal) => State::ValueNull('n'),
                        ('u', State::ValueNull('n')) => State::ValueNull('u'),
                        ('l', State::ValueNull('u')) => State::ValueNull('l'),
                        ('l', State::ValueNull('l')) => {
                            tokens.push(Token::Null);
                            State::Normal
                        }

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

    #[test]
    fn should_lex_error_when_null_interrupted_by_space() {
        run_expected_error_test_case_with(
            "{ \"key\": nu ll}",
            JSONError::new(format!("Unexpected ' '"), 1),
        )
    }

    #[test]
    fn should_lex_error_when_true_interrupted_by_space() {
        run_expected_error_test_case_with(
            "{ \"key\": t  rue}",
            JSONError::new(format!("Unexpected ' '"), 1),
        )
    }

    #[test]
    fn should_lex_error_when_number_starts_with_zero() {
        run_expected_error_test_case_with(
            "{ \"key\": 011}",
            JSONError::new(format!("Unexpected '1'"), 1),
        )
    }

    #[test]
    fn should_lex_correctly_when_number_is_zero() {
        run_test_case_with(
            "{ \"key\": 0}",
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
    fn should_lex_correctly_decimal_number() {
        run_test_case_with(
            "{ \"key\": 1.5}",
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
    fn should_lex_correctly_decimal_number_with_leading_zero() {
        run_test_case_with(
            "{ \"key\": 0.2}",
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
    fn should_lex_error_with_decimal_with_multiple_points() {
        run_expected_error_test_case_with(
            "{ \"key\": 0.1.1}",
            JSONError::new(format!("Unexpected '.'"), 1),
        )
    }

    #[test]
    fn should_lex_error_with_decimal_with_multiple_consecutive_points() {
        run_expected_error_test_case_with(
            "{ \"key\": 0..1}",
            JSONError::new(format!("Unexpected '.'"), 1),
        )
    }

    #[test]
    fn should_lex_error_with_zero_followed_by_null() {
        run_expected_error_test_case_with(
            "{ \"key\": 0null}",
            JSONError::new(format!("Unexpected 'n'"), 1),
        )
    }

    #[test]
    fn should_lex_error_with_zero_followed_by_true() {
        run_expected_error_test_case_with(
            "{ \"key\": 0true}",
            JSONError::new(format!("Unexpected 't'"), 1),
        )
    }

    #[test]
    fn should_lex_error_with_zero_followed_by_false() {
        run_expected_error_test_case_with(
            "{ \"key\": 0false}",
            JSONError::new(format!("Unexpected 'f'"), 1),
        )
    }

    #[test]
    fn should_lex_error_with_zero_followed_by_open_bracket() {
        run_expected_error_test_case_with(
            "{ \"key\": 0[}",
            JSONError::new(format!("Unexpected '['"), 1),
        )
    }

    #[test]
    fn should_lex_correctly_negative_number_leading_zero() {
        run_test_case_with(
            "{ \"key\": -0.2}",
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
    fn should_lex_correctly_negative_number() {
        run_test_case_with(
            "{ \"key\": -1.2}",
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
    fn should_lex_correctly_whitespaces_before_column() {
        run_test_case_with(
            "{ \"key\"  : -1.2}",
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
    fn should_lex_correctly_plus_number() {
        run_test_case_with(
            "{ \"key\": +1.2}",
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
    fn should_lex_correctly_plus_number_before_zero() {
        run_test_case_with(
            "{ \"key\": +0}",
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
    fn should_lex_correctly_exponential_e_after_zero() {
        run_test_case_with(
            "{ \"key\": 0e0}",
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
    fn should_lex_correctly_exponential_capital_e_after_zero() {
        run_test_case_with(
            "{ \"key\": 0E0}",
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
    fn should_lex_correctly_exponential_e_after_int() {
        run_test_case_with(
            "{ \"key\": 1e0}",
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
    fn should_lex_correctly_exponential_e_after_decimal() {
        run_test_case_with(
            "{ \"key\": 1.2e0}",
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
    fn should_lex_correctly_exponential_non_zero_exp() {
        run_test_case_with(
            "{ \"key\": 1.2E2}",
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
    fn should_lex_correctly_exponential_with_plus() {
        run_test_case_with(
            "{ \"key\": 1.2e+2}",
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
    fn should_lex_correctly_exponential_with_minus() {
        run_test_case_with(
            "{ \"key\": 1.2e-10}",
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
    fn should_lex_error_exponential_decimal_exp_with_sign() {
        run_expected_error_test_case_with(
            "{ \"key\": 1.2E+1.2}",
            JSONError::new("Unexpected '.'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_exponential_decimal_exp_leading_zero_with_sign() {
        run_expected_error_test_case_with(
            "{ \"key\": 1.2E-0.2}",
            JSONError::new("Unexpected '.'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_exponential_decimal_exp_leading_zero() {
        run_expected_error_test_case_with(
            "{ \"key\": 1.2E0.2}",
            JSONError::new("Unexpected '.'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_exponential_decimal_exp() {
        run_expected_error_test_case_with(
            "{ \"key\": 1.2e2.4}",
            JSONError::new("Unexpected '.'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_repeated_exp() {
        run_expected_error_test_case_with(
            "{ \"key\": 1e0e0}",
            JSONError::new("Unexpected 'e'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_plus_followed_by_brace() {
        run_expected_error_test_case_with(
            "{ \"key\": +}",
            JSONError::new("Unexpected '}'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_minus_followed_by_brace() {
        run_expected_error_test_case_with(
            "{ \"key\": -}",
            JSONError::new("Unexpected '}'".to_string(), 1),
        )
    }
    #[test]
    fn should_lex_error_with_plus_followed_by_comma() {
        run_expected_error_test_case_with(
            "{ \"key\": +,\"\":0}",
            JSONError::new("Unexpected ','".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_minus_followed_by_newline() {
        run_expected_error_test_case_with(
            "{ \"key\": -\n}",
            JSONError::new("Unexpected '\n'".to_string(), 1),
        )
    }
    #[test]
    fn should_lex_error_with_plus_followed_by_newline() {
        run_expected_error_test_case_with(
            "{ \"key\": +\n}",
            JSONError::new("Unexpected '\n'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_minus_followed_by_comma() {
        run_expected_error_test_case_with(
            "{ \"key\": -,\"\":0}",
            JSONError::new("Unexpected ','".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_e_followed_by_brace() {
        run_expected_error_test_case_with(
            "{ \"key\": 0e}",
            JSONError::new("Unexpected '}'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_capital_e_followed_by_brace() {
        run_expected_error_test_case_with(
            "{ \"key\": 0E}",
            JSONError::new("Unexpected '}'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_point_followed_by_brace() {
        run_expected_error_test_case_with(
            "{ \"key\": 0.}",
            JSONError::new("Unexpected '}'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_point_followed_by_comma() {
        run_expected_error_test_case_with(
            "{ \"key\": 0.,\"\":0}",
            JSONError::new("Unexpected ','".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_point_followed_by_return() {
        run_expected_error_test_case_with(
            "{ \"key\": 0.\n,\"\":0}",
            JSONError::new("Unexpected '\n'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_exp_followed_by_comma() {
        run_expected_error_test_case_with(
            "{ \"key\": 0e,\"\":0}",
            JSONError::new("Unexpected ','".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_error_with_exp_followed_by_return() {
        run_expected_error_test_case_with(
            "{ \"key\": 0e\n,\"\":0}",
            JSONError::new("Unexpected '\n'".to_string(), 1),
        )
    }

    #[test]
    fn should_lex_correctly_array_with_zero() {
        run_test_case_with(
            "[0]",
            Vec::from([Token::OpenBracket, Token::Number, Token::ClosedBracket]),
        )
    }

    #[test]
    fn should_lex_correctly_array_with_one() {
        run_test_case_with(
            "[1]",
            Vec::from([Token::OpenBracket, Token::Number, Token::ClosedBracket]),
        )
    }

    #[test]
    fn should_lex_correctly_array_with_decimal() {
        run_test_case_with(
            "[1,2.6]",
            Vec::from([
                Token::OpenBracket,
                Token::Number,
                Token::Comma,
                Token::Number,
                Token::ClosedBracket,
            ]),
        )
    }

    #[test]
    fn should_lex_correctly_array_with_exp() {
        run_test_case_with(
            "[1,2.6e9]",
            Vec::from([
                Token::OpenBracket,
                Token::Number,
                Token::Comma,
                Token::Number,
                Token::ClosedBracket,
            ]),
        )
    }

    #[test]
    fn should_lex_correctly_array_with_zero_exp() {
        run_test_case_with(
            "[1,2.6e0]",
            Vec::from([
                Token::OpenBracket,
                Token::Number,
                Token::Comma,
                Token::Number,
                Token::ClosedBracket,
            ]),
        )
    }
}
