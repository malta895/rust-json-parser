use super::{error::JSONError, token::Token};

struct State {
    state: StateKind,

    obj_depth: u64,
}

impl State {
    pub fn new() -> State {
        State {
            state: StateKind::Initial,
            obj_depth: 0,
        }
    }

    fn close_obj(&mut self) {
        self.obj_depth -= 1;
        self.state = if self.obj_depth == 0 {
            StateKind::End
        } else {
            StateKind::AfterObj
        };
    }

    fn open_obj(&mut self) {
        self.state = StateKind::OpenObj;
        self.obj_depth += 1;
    }
}

#[derive(Debug, PartialEq)]
enum StateKind {
    Initial,

    OpenObj,

    Key,
    Val,
    AfterVal,
    AfterComma,

    AfterObj,

    End,
}

pub fn parse(tokens: Vec<Token>) -> Result<(), JSONError> {
    let mut state = State::new();
    for token in &tokens {
        match (&state.state, token) {
            (_, Token::NewLine) => {}
            (StateKind::Initial, Token::OpenBrace) => {
                state.open_obj();
            }
            (StateKind::Initial, Token::ClosedBrace) => {
                if state.obj_depth == 0 {
                    return Err(JSONError::new("Unexpected '}'".to_string(), 1));
                }
            }

            (StateKind::End, _) => {
                return Err(JSONError::new(format!("Unexpected <string literal>"), 1));
            }

            (StateKind::OpenObj, Token::StringLiteral(_)) => {
                state.state = StateKind::Key;
            }

            (StateKind::OpenObj, Token::ClosedBrace) => {
                state.close_obj();
            }

            (StateKind::AfterObj, Token::ClosedBrace) => {
                state.close_obj();
            }
            (StateKind::AfterObj, Token::Comma) => {
                state.state = StateKind::AfterComma;
            }

            (StateKind::Key, Token::Column) => {
                state.state = StateKind::Val;
            }
            (
                StateKind::Val,
                Token::StringLiteral(_)
                | Token::BoolFalse
                | Token::BoolTrue
                | Token::Null
                | Token::Number(_),
            ) => {
                state.state = StateKind::AfterVal;
            }
            (StateKind::Val, Token::OpenBrace) => {
                state.open_obj();
            }

            (StateKind::AfterVal, Token::ClosedBrace) => {
                state.close_obj();
            }
            (StateKind::AfterVal, Token::Comma) => {
                state.state = StateKind::AfterComma;
            }

            (StateKind::AfterComma, Token::StringLiteral(_)) => {
                state.state = StateKind::Key;
            }

            (_, token) => {
                return Err(JSONError::new(format!("Unexpected {}", token), 1));
            }
        }
    }
    if state.state != StateKind::End {
        dbg!(state.state);
        return Err(JSONError::new(format!("Unexpected EOF"), 1));
    }
    Ok(())
}

#[cfg(test)]
mod test_parser_pass {
    use crate::parser::token::Token;
    macro_rules! test_parser_passes {
        ($($name:ident: $value:expr,)*) => {
            use super::parse;
            $(
                #[test]
                fn $name() {
                    let input = $value;
                    assert_eq!((), parse(input).unwrap());
                }
            )*
            }
    }

    test_parser_passes! {
        with_closed_open_brace: vec![Token::OpenBrace, Token::ClosedBrace],
        with_base_key_literal_object: vec![
            Token::OpenBrace,
            Token::StringLiteral("key".to_string()),
            Token::Column,
            Token::StringLiteral("value".to_string()),
            Token::ClosedBrace
        ],
        with_inner_empty_object: vec![
            Token::OpenBrace,
            Token::StringLiteral("key".to_string()),
            Token::Column,
            Token::OpenBrace,
            Token::ClosedBrace,
            Token::ClosedBrace
        ],
        with_inner_object: vec![
            Token::OpenBrace, // {
            Token::StringLiteral("key".to_string()), // "key"
            Token::Column, // :
            Token::OpenBrace, // {
            Token::StringLiteral("key".to_string()), // "key"
            Token::Column, // :
            Token::OpenBrace, // {
            Token::ClosedBrace, // }
            Token::ClosedBrace, // }
            Token::ClosedBrace // }
        ],
        ignores_new_line: vec![
            Token::OpenBrace, // {
            Token::StringLiteral("key".to_string()), // "key"
            Token::NewLine,
            Token::Column, // :
            Token::OpenBrace, // {
            Token::StringLiteral("key".to_string()), // "key"
            Token::Column, // :
            Token::OpenBrace, // {
            Token::ClosedBrace, // }
            Token::ClosedBrace, // }
            Token::ClosedBrace // }
        ],
        obj_with_multiple_values: vec![
            Token::OpenBrace,
            Token::StringLiteral("key".to_string()),
            Token::Column,
            Token::StringLiteral("value".to_string()),
            Token::Comma,
            Token::StringLiteral("key2".to_string()),
            Token::Column,
            Token::StringLiteral("value2".to_string()),
            Token::ClosedBrace,
        ],
        consider_any_possible_kind_of_value: vec![
            Token::OpenBrace,
            Token::NewLine,
            Token::StringLiteral("key1".to_string()),
            Token::Column,
            Token::BoolTrue,
            Token::NewLine,
            Token::Comma,
            Token::StringLiteral("key2".to_string()),
            Token::Column,
            Token::BoolFalse,
            Token::NewLine,
            Token::Comma,
            Token::StringLiteral("key3".to_string()),
            Token::Column,
            Token::Null,
            Token::NewLine,
            Token::Comma,
            Token::StringLiteral("key4".to_string()),
            Token::Column,
            Token::StringLiteral("value".to_string()),
            Token::NewLine,
            Token::Comma,
            Token::StringLiteral("key5".to_string()),
            Token::Column,
            Token::Number(101.),
            Token::NewLine,
            Token::ClosedBrace,
        ],
    }
}

#[cfg(test)]
mod test_parser_failure {
    use crate::parser::{error::JSONError, token::Token};
    macro_rules! test_parser_fails {
        ($($name:ident: $value:expr,)*) => {
            use super::parse;
            $(
                #[test]
                fn $name() {
                    let (input, expected_err) = $value;
                    assert_eq!(expected_err, parse(input).unwrap_err());
                }
            )*
            }
    }

    test_parser_fails! {
        with_only_open_brace: (
            vec![Token::OpenBrace],
            JSONError::new("Unexpected EOF".to_string(), 1),
        ),
        with_no_tokens: (
            vec![],
            JSONError::new("Unexpected EOF".to_string(), 1),
        ),
        with_only_closed_brace: (
            vec![Token::ClosedBrace],
            JSONError::new("Unexpected '}'".to_string(), 1),
        ),
        literal_outside_obj: (
            vec![
                Token::OpenBrace,
                Token::StringLiteral("key".to_string()),
                Token::Column,
                Token::StringLiteral("value".to_string()),
                Token::ClosedBrace,
                Token::StringLiteral("outsider value".to_string()),
            ],
            JSONError::new("Unexpected <string literal>".to_string(), 1),
        ),
        with_closure_after_comma:(
            vec![
                Token::OpenBrace,
                Token::StringLiteral("key".to_string()),
                Token::Column,
                Token::OpenBrace,
                Token::StringLiteral("key".to_string()),
                Token::Column,
                Token::OpenBrace,
                Token::ClosedBrace,
                Token::ClosedBrace,
                Token::Comma,
                Token::ClosedBrace
            ],
            JSONError::new("Unexpected '}'".to_string(), 1),
        ),
    }
}
