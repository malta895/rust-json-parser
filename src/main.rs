use std::{io::BufReader, process::exit};

mod parser;

fn main() {
    let buf = BufReader::new(std::io::stdin());

    match parser::JSONParser::check_valid(buf) {
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
        Ok(()) => {
            println!("ok")
        }
    }
}
