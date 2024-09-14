use std::os::linux::raw::stat;

use super::{error::JSONError, token::Token};

struct State {
    state_kind: StateKind,

    obj_depth: u64,
    array_depth: u64,
}

impl State {
    pub fn new() -> State {
        State {
            state_kind: StateKind::Initial,
            obj_depth: 0,
            array_depth: 0,
        }
    }

    fn close_obj(&mut self) {
        self.obj_depth -= 1;
        self.state_kind = if self.obj_depth == 0 {
            StateKind::End
        } else {
            StateKind::AfterObj
        };
    }

    fn open_obj(&mut self) {
        self.state_kind = StateKind::OpenObj;
        self.obj_depth += 1;
    }
}

#[derive(Debug, PartialEq)]
enum StateKind {
    Initial,

    OpenObj,
    OpenArr,

    ObjKey,
    ObjVal,
    AfterObjVal,
    ObjComma,

    ArrVal,

    AfterObj,

    End,
}

pub fn parse(tokens: Vec<Token>) -> Result<(), JSONError> {
    let mut state = State::new();
    for token in &tokens {
        match (&state.state_kind, token) {
            (_, Token::NewLine) => {}
            (StateKind::Initial, Token::OpenBrace) => {
                state.open_obj();
            }
            (StateKind::Initial, Token::ClosedBrace) => {
                return Err(JSONError::new("Unexpected '}'".to_string(), 1));
            }
            (StateKind::Initial, Token::OpenBracket) => {
                //state.array_depth += 1;
                state.state_kind = StateKind::OpenArr;
            }

            (StateKind::End, _) => {
                return Err(JSONError::new(format!("Unexpected <string literal>"), 1));
            }

            (StateKind::OpenObj, Token::StringLiteral(_)) => {
                state.state_kind = StateKind::ObjKey;
            }
            (StateKind::OpenObj, Token::ClosedBrace) => {
                state.close_obj();
            }

            (StateKind::AfterObj, Token::ClosedBrace) => {
                state.close_obj();
            }
            (StateKind::AfterObj, Token::Comma) => {
                state.state_kind = StateKind::ObjComma;
            }

            (StateKind::OpenArr, Token::ClosedBracket) => {
                //state.array_depth -= 1;
                state.state_kind = StateKind::End;
            }
            (
                StateKind::OpenArr,
                Token::StringLiteral(_)
                | Token::BoolFalse
                | Token::BoolTrue
                | Token::Null
                | Token::Number(_),
            ) => {
                state.state_kind = StateKind::ArrVal;
            }

            (StateKind::ObjKey, Token::Column) => {
                state.state_kind = StateKind::ObjVal;
            }

            (StateKind::ArrVal, Token::ClosedBracket) => state.state_kind = StateKind::End,
            (StateKind::ArrVal, Token::Comma) => state.state_kind = StateKind::OpenArr,

            (
                StateKind::ObjVal,
                Token::StringLiteral(_)
                | Token::BoolFalse
                | Token::BoolTrue
                | Token::Null
                | Token::Number(_),
            ) => {
                state.state_kind = StateKind::AfterObjVal;
            }
            (StateKind::ObjVal, Token::OpenBrace) => {
                state.open_obj();
            }

            (StateKind::AfterObjVal, Token::ClosedBrace) => {
                state.close_obj();
            }
            (StateKind::AfterObjVal, Token::Comma) => {
                state.state_kind = StateKind::ObjComma;
            }

            (StateKind::ObjComma, Token::StringLiteral(_)) => {
                state.state_kind = StateKind::ObjKey;
            }

            (_, token) => {
                return Err(JSONError::new(format!("Unexpected {}", token), 1));
            }
        }
    }
    if state.state_kind != StateKind::End {
        dbg!(state.state_kind);
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
        empty_array: vec![
            Token::OpenBracket,
            Token::ClosedBracket
        ],

        array_with_one_value: vec![
            Token::OpenBracket,
            Token::Null,
            Token::ClosedBracket
        ],
        array_with_all_possible_values: vec![
            Token::OpenBracket,
            Token::Null,
            Token::Comma,
            Token::BoolTrue,
            Token::Comma,
            Token::BoolFalse,
            Token::Comma,
            Token::StringLiteral("some value".to_string()),
            Token::Comma,
            Token::Number(1.),
            Token::ClosedBracket
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
