use super::{error::JSONError, token::Token};

#[derive(Debug)]
struct State {
    state_kind: StateKind,
    obj_arr_stack: Vec<Stack>,
}

impl State {
    pub fn new() -> State {
        State {
            state_kind: StateKind::Initial,
            obj_arr_stack: vec![],
        }
    }

    fn close_obj(&mut self) -> Result<(), JSONError> {
        self.state_kind = match self.obj_arr_stack.pop() {
            Some(Stack::Object) if self.obj_arr_stack.len() == 0 => StateKind::End,
            Some(Stack::Object) => StateKind::AfterObjVal,
            Some(_) | None => return Err(JSONError::new("Unexpected '}'".to_string(), 1)),
        };
        Ok(())
    }

    fn open_obj(&mut self) {
        self.state_kind = StateKind::OpenObj;
        self.obj_arr_stack.push(Stack::Object)
    }

    fn open_arr(&mut self) {
        self.state_kind = StateKind::OpenArr;
        self.obj_arr_stack.push(Stack::Array)
    }

    fn close_arr(&mut self) -> Result<(), JSONError> {
        self.state_kind = match self.obj_arr_stack.pop() {
            Some(Stack::Array) if self.obj_arr_stack.len() == 0 => StateKind::End,
            Some(Stack::Array) => StateKind::ArrVal,
            Some(_) | None => return Err(JSONError::new("Unexpected ']'".to_string(), 1)),
        };
        Ok(())
    }

    fn is_inside_object(&self) -> bool {
        match self.obj_arr_stack.last() {
            Some(Stack::Object) => true,
            Some(_) | None => false,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Stack {
    Object,
    Array,
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
    ArrValAfterComma,

    End,
}

pub fn parse(tokens: Vec<Token>) -> Result<(), JSONError> {
    let mut state = State::new();
    for token in &tokens {
        dbg!("processing:", token, &state);
        match (&state.state_kind, token) {
            (_, Token::NewLine) => {}
            (StateKind::Initial, Token::Null) => {
                state.state_kind = StateKind::End;
            }
            (StateKind::Initial, Token::OpenBrace) => {
                state.open_obj();
            }
            (StateKind::Initial, Token::ClosedBrace) => {
                return Err(JSONError::new("Unexpected '}'".to_string(), 1));
            }
            (StateKind::Initial, Token::OpenBracket) => {
                state.open_arr();
            }

            (StateKind::End, token) => {
                dbg!("state end with token", token);
                return Err(JSONError::new(format!("Unexpected {}", token), 1));
            }

            (StateKind::OpenObj, Token::StringLiteral(_)) => {
                state.state_kind = StateKind::ObjKey;
            }
            (StateKind::OpenObj, Token::ClosedBrace) => {
                state.close_obj()?;
            }

            (StateKind::OpenArr, Token::ClosedBracket) => {
                state.close_arr()?;
            }
            (StateKind::OpenArr, Token::OpenBracket) => {
                state.open_arr();
            }
            (StateKind::OpenArr, Token::OpenBrace) => {
                state.open_obj();
            }
            (StateKind::ArrValAfterComma, Token::StringLiteral(_)) if state.is_inside_object() => {
                state.state_kind = StateKind::ObjKey;
            }

            (
                StateKind::OpenArr | StateKind::ArrValAfterComma,
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

            (StateKind::ArrVal, Token::ClosedBracket) => {
                state.close_arr()?;
            }
            (StateKind::ArrValAfterComma, Token::OpenBrace) => {
                state.open_obj();
            }
            (StateKind::ArrValAfterComma, Token::OpenBracket) => {
                state.open_arr();
            }
            (StateKind::ArrVal, Token::ClosedBrace) => {
                state.close_obj()?;
            }
            (StateKind::ArrVal, Token::Comma) => state.state_kind = StateKind::ArrValAfterComma,

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
            (StateKind::ObjVal, Token::OpenBracket) => {
                state.open_arr();
            }

            (StateKind::AfterObjVal, Token::ClosedBrace) => {
                state.close_obj()?;
            }
            (StateKind::AfterObjVal, Token::ClosedBracket) => {
                state.close_arr()?;
            }
            (StateKind::AfterObjVal, Token::Comma) => match state.obj_arr_stack.last() {
                Some(Stack::Array) => {
                    state.state_kind = StateKind::ArrValAfterComma;
                }
                Some(Stack::Object) => {
                    state.state_kind = StateKind::ObjComma;
                }
                None => (),
            },

            (StateKind::ObjComma, Token::StringLiteral(_)) => {
                state.state_kind = StateKind::ObjKey;
            }

            (_, token) => {
                dbg!("unexpected kind", token, state);
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
        array_with_nested_array: vec![
            Token::OpenBracket,
            Token::OpenBracket,
            Token::StringLiteral("nested".to_string()),
            Token::ClosedBracket,
            Token::ClosedBracket
        ],
        array_with_nested_empty_array: vec![
            Token::OpenBracket,
            Token::OpenBracket,
            Token::ClosedBracket,
            Token::ClosedBracket
        ],
        empty_array_as_obj_value: vec![
            Token::OpenBrace,
            Token::StringLiteral("key".to_string()),
            Token::Column,
            Token::OpenBracket,
            Token::ClosedBracket,
            Token::ClosedBrace,
        ],
        array_with_string_and_object: vec![
            Token::OpenBracket,
            Token::StringLiteral("some string".to_string()),
            Token::Comma,
            Token::OpenBrace,
            Token::ClosedBrace,
            Token::ClosedBracket,
        ],
        array_with_object_with_element: vec![
            Token::OpenBracket,//[
            Token::StringLiteral("some string".to_string()),//""
            Token::Comma,//,
            Token::OpenBrace,//{
            Token::StringLiteral("some key".to_string()),//""
            Token::Column,//:
            Token::OpenBracket,//[
            Token::StringLiteral("some string".to_string()),//""
            Token::ClosedBracket,//]
            Token::ClosedBrace, //}
            Token::Comma,//,
            Token::OpenBrace,//{
            Token::ClosedBrace,//}
            Token::ClosedBracket,
        ],
        object_as_first_array_element: vec![
            Token::OpenBracket,
            Token::NewLine,
            Token::OpenBrace,
            Token::StringLiteral("key".to_string()),
            Token::Column,
            Token::OpenBracket,
            Token::StringLiteral("value".to_string()),
            Token::ClosedBracket,
            Token::ClosedBrace,
            Token::NewLine,
            Token::ClosedBracket,
        ],
        lonely_null: vec![Token::Null],
        array_after_arr_value: vec![
            Token::OpenBracket,
            Token::StringLiteral("some value".to_string()),
            Token::Comma,
            Token::OpenBracket,
            Token::ClosedBracket,
            Token::ClosedBracket,
        ],
        empty_array_followed_by_empty_obj: vec![
            Token::OpenBrace,
            Token::StringLiteral("key".to_string()),
            Token::Column,
            Token::OpenBracket,
            Token::ClosedBracket,
            Token::Comma,
            Token::StringLiteral("object_key".to_string()),
            Token::Column,
            Token::StringLiteral("some value".to_string()),
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
            JSONError::new("Unexpected '<string literal>'".to_string(), 1),
        ),
        true_outside_obj: (
            vec![
                Token::OpenBrace,
                Token::ClosedBrace,
                Token::BoolTrue,
            ],
            JSONError::new("Unexpected '<boolean>'".to_string(), 1),
        ),
        false_outside_obj: (
            vec![
                Token::OpenBrace,
                Token::ClosedBrace,
                Token::BoolFalse,
            ],
            JSONError::new("Unexpected '<boolean>'".to_string(), 1),
        ),
        null_outside_obj: (
            vec![
                Token::OpenBrace,
                Token::ClosedBrace,
                Token::Null,
            ],
            JSONError::new("Unexpected '<null>'".to_string(), 1),
        ),
        number_outside_obj: (
            vec![
                Token::OpenBrace,
                Token::ClosedBrace,
                Token::Number(0.),
            ],
            JSONError::new("Unexpected '<number>'".to_string(), 1),
        ),
        with_closure_after_comma: (
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
        with_extra_comma_in_array: (
            vec![
                Token::OpenBracket,
                Token::StringLiteral("hello".to_string()),
                Token::Comma,
                Token::ClosedBracket,
            ],
            JSONError::new("Unexpected ']'".to_string(), 1),
        ),
    }
}
