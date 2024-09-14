use super::{error::JSONError, token::Token};

pub fn parse(tokens: Vec<Token>)-> Result<(), JSONError>{
    if tokens.is_empty() || tokens.len() == 1 && tokens[0] == Token::OpenBrace {
        return Err(JSONError::new("Unexpected EOF".to_string(), 1));
    }
    Ok(())
}

#[cfg(test)]
mod test_parser_pass{
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
    }
}

#[cfg(test)]
mod test_parser_failure {
    use crate::parser::{token::Token, error::JSONError};
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
    }
}

