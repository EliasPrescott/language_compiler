mod parsing;
mod parsers;
mod AST;

use std::fs;
use AST::*;

const TESTING_FILE_PATH: &str = "test.txt";

fn main() {
    match fs::read_to_string(TESTING_FILE_PATH) {
        Ok(contents) => {

            let ast_tree = parse_ast_text(contents);

            println!("{:#?}", ast_tree);
        },
        Err(err) => println!("{}", err)        
    }
}
