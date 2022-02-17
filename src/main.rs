mod parsing;

use std::fs;
use parsing::*;

const TESTING_FILE_PATH: &str = "test.txt";

type ASTNameRaw = (String);

#[derive(Debug)]
enum ASTExpression {
    ASTVariableRef(String),
    ASTInteger(i64),
    ASTAssignment(String, Box<ASTExpression>),
    Unit,
}

fn parse_integer(input: &mut ParseInput) -> Result<i64, String> {
    let first_char = input.get_next_char_numerical()?;
    let mut output = first_char.to_string();
    input.skip_next_char();
    while let Ok(next_char) = input.get_next_char_numerical() {
        output += &next_char.to_string();
        input.skip_next_char();
    }
    Ok(str::parse::<i64>(&output).map_err(|err| err.to_string())?)
}

fn parse_name(input: &mut ParseInput) -> Result<String, String> {
    let accepted_nonpreceding_symbols = vec!('_', '-');
    let first_char = input.pop_next_char_alphabetical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_alphabetical_or_in_group(&accepted_nonpreceding_symbols) {
        output += &next_char.to_string();
    }
    Ok(output)
}

fn parse_variable_ref(input: &mut ParseInput) -> Result<ASTExpression, String> {
    Ok(ASTExpression::ASTVariableRef(parse_name(input)?))
}

fn parse_integer_assignment(input: &mut ParseInput) -> Result<ASTExpression, String> {
    input.skip_string("let")?;
    input.skip_spaces_and_newlines();
    let variable_name = parse_name(input)?;
    input.skip_spaces_and_newlines();
    input.skip_char('=')?;
    input.skip_spaces_and_newlines();
    let variable_value = parse_integer(input)?;
    input.skip_char(';')?;
    Ok(ASTExpression::ASTAssignment(variable_name, Box::new(ASTExpression::ASTInteger(variable_value))))
}

/// Tries every parser in a list. Returns the first successful parse result, or the last error if all fail. 
fn try_parsers<T>(input: &mut ParseInput, parsers: Vec<&dyn Fn(&mut ParseInput) -> Result<T, String>>) -> Result<T, String> {
    let save_point = input.create_save_point();
    let mut last_err = String::new();
    for parser in parsers {
        match parser(input) {
            Ok(x) => return Ok(x),
            Err(err) => {
                last_err = err;
                input.load_save_point(save_point);
            }
        }
    }
    Err(last_err.to_string())
}

fn main() {
    match fs::read_to_string(TESTING_FILE_PATH) {
        Ok(contents) => {
            let mut input = ParseInput::new(contents);
            loop {
                if !input.finished() {
                    input.skip_spaces_and_newlines();
                    match try_parsers(&mut input, vec!(&parse_integer_assignment, &parse_variable_ref)) {
                        Ok(expr) => {
                            println!("{:?}", expr);
                        },
                        Err(err) => {
                            eprintln!("{}", err);
                            break;
                        }
                    }
                } else {
                    break;
                }
            }            
        },
        Err(err) => println!("{}", err)
    }
}