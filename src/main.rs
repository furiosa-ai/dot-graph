extern crate dot_parser;

use dot_parser::parser::parse;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    println!("{:?}", parse(&path));
}
