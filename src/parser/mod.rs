use std::io::{BufRead, Read};

mod token;
use lexer::lex;
use parser::parse;
use token::Token;

mod error;
use error::JSONError;

mod lexer;
mod parser;

pub fn check_valid<R: BufRead>(reader: R) -> Result<(), JSONError> {
    let tokens = lex(reader)?;
    parse(tokens)
}

#[cfg(test)]
mod check_valid_tests {
    use crate::parser::check_valid;

    #[test]
    fn should_not_report_error_for_obj() {
        let res = check_valid("{}".as_bytes());
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn should_report_error_for_not_closed_brace() {
        let found_err = check_valid("{".as_bytes()).unwrap_err();
        assert_eq!("Unexpected EOF: at line 1", found_err.to_string())
    }

    #[test]
    fn should_report_error_for_closed_brace_outside_obj() {
        let found_err = check_valid("}".as_bytes()).unwrap_err();
        assert_eq!(
            "Unexpected '}' outside obj: at line 1",
            found_err.to_string()
        )
    }

    #[test]
    fn should_report_error_for_random_char() {
        let found_err = check_valid("{a".as_bytes()).unwrap_err();
        assert_eq!("Unexpected 'a': at line 1", found_err.to_string())
    }

    #[test]
    fn should_report_unexpected_eof_for_empty_file() {
        let found_err = check_valid("".as_bytes()).unwrap_err();
        assert_eq!("Unexpected EOF: at line 1", found_err.to_string())
    }

    #[test]
    fn should_not_report_error_for_new_line_at_the_end_of_file() {
        let res = check_valid("{}\n".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_base_case() {
        let res = check_valid("{\"\":\"\"}\n".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_base_case_with_key_val() {
        let res = check_valid("{\"key\":\"value\"}\n".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_string_with_spaces() {
        let res = check_valid("{  \"key\":\"va l\",\n  \"ke y2\":\"val\"}".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_number() {
        let res = check_valid("{  \"key\": 123}".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_true() {
        let res = check_valid("{  \"key\": true}".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_false() {
        let res = check_valid("{  \"key\": false}".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_null() {
        let res = check_valid("{  \"key\": null}".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_empty_array() {
        let res = check_valid("{  \"key\": []}".as_bytes());
        assert_eq!(Ok(()), res)
    }

    #[test]
    fn should_recognize_nested_objects() {
        let res =
            check_valid("{ \"key\": {\n\"inner_key\":\"inner_val\"\n}\n}".as_bytes());
        assert_eq!(Ok(()), res)
    }
}
