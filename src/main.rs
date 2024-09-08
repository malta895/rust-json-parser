use std::io::{BufReader, Read};

use rust_json_parser::JSONParser;

fn main() {
    let buf = BufReader::new(std::io::stdin());

    match JSONParser::check_valid(buf) {
        Err(e) => {
            eprintln!("{}", e);
        }
        Ok(()) => {
            println!("ok")
        }
    }
}
