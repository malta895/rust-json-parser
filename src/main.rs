use std::io::BufReader;

mod parser;

fn main() {
    let buf = BufReader::new(std::io::stdin());

    match parser::JSONParser::check_valid(buf) {
        Err(e) => {
            eprintln!("{}", e);
        }
        Ok(()) => {
            println!("ok")
        }
    }
}
