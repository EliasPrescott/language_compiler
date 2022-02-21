mod parsing;

use std::{fs, rc::Rc, cell::RefCell};
use parsing::*;

const TESTING_FILE_PATH: &str = "test.txt";

#[derive(Debug)]
enum ASTExpression {
    ASTVariableRef(String),
    ASTInteger(i64),

    // I'm only parsing integer assignment right now, but this variant is ready for future use by accepting all expression types.
    ASTAssignment(String, Box<ASTExpression>),
    ASTScope(Vec<Box<ASTExpression>>),
    ASTUnit,
}

fn parse_integer(input: &mut ParseInput) -> Result<i64, String> {
    // The pop_next_char_numerical and other similar methods only mutate the ParseInput if the next char matches the predicate.
    // This fact will need to be explicit in the documentation for these methods.
    let first_char = input.pop_next_char_numerical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_numerical() {
        output += &next_char.to_string();
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

// Since this pattern is so permissive, it is important to put it after other, stricter patterns.
// It could be good to make a list of language keywords (e.g. 'let') and forbid them from being parsed as variable names.
fn parse_variable_ref(input: &mut ParseInput) -> Result<ASTExpression, String> {
    Ok(ASTExpression::ASTVariableRef(parse_name(input)?))
}

// Instead of parsing input directly, this function takes in an interior_parser, and then builds a new parser that will continually run that interior_parser within a braced scope.
fn parse_scope_with_parser(interior_parser: Box<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>> {
    Box::new(move | input: &mut ParseInput | {
        let mut output = vec!();
        input.skip_char('{')?;
        loop {
            input.skip_spaces_and_newlines();
            match interior_parser(input) {
                Ok(x) => {
                    output.push(Box::new(x));
                },
                Err(e) => {
                    if let Ok(()) = input.skip_char('}') {
                        return Ok(ASTExpression::ASTScope(output));
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    })
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
fn try_parsers_with_list<'a>(parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String>>>>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String> + 'a> {
    Box::new(move | input: &mut ParseInput | -> Result<ASTExpression, String> {
        let save_point = input.create_save_point();
        let mut last_err = String::new();
        for parser in parsers.borrow().iter() {
            match parser(input) {
                Ok(x) => return Ok(x),
                Err(err) => {
                    last_err = err;
                    input.load_save_point(save_point);
                }
            }
        }
        Err(last_err.to_string())
    })
}

fn main() {
    match fs::read_to_string(TESTING_FILE_PATH) {
        Ok(contents) => {
            let parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String>>>>> = Rc::new(RefCell::new(vec!(
                Box::new(parse_integer_assignment), 
                Box::new(parse_variable_ref),
            )));

            // I don't think I should need to make this parser twice, but I'm how it could be shared properly.
            // Building it a few times likely adds very little overhead, so it's probably not a huge concern.
            let main_parser = try_parsers_with_list(parsers.clone());
            let main_parser_ref = try_parsers_with_list(parsers.clone());

            let scope_parser = parse_scope_with_parser(Box::new(main_parser_ref));

            // After constructing the scope_parser and passing the main parser into it, I then add the scope_parser into the main parser.
            // This allows for endless recursive parsing, but it also makes the type definitions explode in length.
            // A Rc<RefCell<Vec<Box<...>>>> doesn't exactly roll off the tongue.
            parsers.borrow_mut().push(Box::new(scope_parser));

            let mut input = ParseInput::new(contents);

            let mut ast_tree: Vec<ASTExpression> = Vec::new();

            // The below loop should probably go into a parse_file function somewhere.
            // If I make a Parser type for my parsing functions, maybe it could live in that type as a static method.
            loop {
                if !input.finished() {
                    input.skip_spaces_and_newlines();
                    match main_parser(&mut input) {
                        Ok(expr) => {
                            ast_tree.push(expr);
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

            // Only just learned this, but :#? allows for pretty-printing the AST output.
            println!("{:#?}", ast_tree);
        },
        Err(err) => println!("{}", err)        
    }
}