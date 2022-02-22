mod parsing;

use std::{fs, rc::Rc, cell::RefCell, sync::Arc};
use parsing::*;

const TESTING_FILE_PATH: &str = "test.txt";

#[derive(Debug)]
enum ASTExpression {
    ASTVariableRef(String),
    ASTInteger(i64),
    ASTString(String),
    ASTAssignment(String, Box<ASTExpression>),
    ASTInitialization(String, Box<ASTExpression>),
    ASTScope(Vec<Box<ASTExpression>>),
    ASTParentheses(Vec<Box<ASTExpression>>),
    ASTFunction(Box<ASTExpression>, Box<ASTExpression>),
    ASTUnit,
}

fn parse_integer(input: &mut ParseInput) -> Result<ASTExpression, String> {
    // The pop_next_char_numerical and other similar methods only mutate the ParseInput if the next char matches the predicate.
    // This fact will need to be explicit in the documentation for these methods.
    let first_char = input.pop_next_char_numerical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_numerical() {
        output += &next_char.to_string();
    }
    Ok(ASTExpression::ASTInteger(str::parse::<i64>(&output).map_err(|err| err.to_string())?))
}

fn parse_string_literal(input: &mut ParseInput) -> Result<ASTExpression, String> {
    // The question mark operator at the end of these parser functions will return early if that expression is a Result::Err case.
    // This allows for extremely concise and convenient error-handling.
    input.skip_char('"')?;
    let output = input.pop_until_char('"');
    input.skip_char('"')?;
    Ok(ASTExpression::ASTString(output))
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
fn parse_scope_with_parser(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>> {
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

fn parse_parentheses_with_parser(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>> {
    Box::new(move | input: &mut ParseInput | {
        let mut output = vec!();
        input.skip_char('(')?;
        loop {
            input.skip_spaces_and_newlines();
            match interior_parser(input) {
                Ok(x) => {
                    output.push(Box::new(x));
                },
                Err(e) => {
                    if let Ok(()) = input.skip_char(')') {
                        return Ok(ASTExpression::ASTParentheses(output));
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    })
}

fn parse_parentheses_with_parsers<'a>(parsers: Box<Vec<&'a (dyn for<'r> Fn(&'r mut parsing::ParseInput) -> Result<ASTExpression, String> + 'a)>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String> + 'a> {
    Box::new(move | input: &mut ParseInput | {
        let mut output = vec!();
        input.skip_char('(')?;
        while input.skip_char(')').is_err() {
            output.push(Box::new(try_parsers(input, parsers.to_vec())?));
        }
        Ok(ASTExpression::ASTParentheses(output))
    })
}

fn parse_assignment_with_parser<'a>(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String> + 'a> {
    Box::new(move | input: &mut ParseInput | {
        let initialization = input.skip_string("let").is_ok();
        input.skip_spaces_and_newlines();
        let variable_name = parse_name(input)?;
        input.skip_spaces_and_newlines();
        input.skip_char('=')?;
        input.skip_spaces_and_newlines();
        let variable_value = interior_parser(input)?;

        if initialization {
            Ok(ASTExpression::ASTInitialization(variable_name, Box::new(variable_value)))
        } else {
            Ok(ASTExpression::ASTAssignment(variable_name, Box::new(variable_value)))
        }
    })
}

// A simpler version of try_parsers that does not support adding in more parsers later for recursion purposes
fn try_parsers(input: &mut ParseInput, parsers: Vec<&dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>>) -> Result<ASTExpression, String> {
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

/// Tries every parser in a list. Returns the first successful parse result, or the last error if all fail. 
fn try_parsers_with_list<'a>(parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String>>>>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String> + 'a> {
    Box::new(move | input: &mut ParseInput | -> Result<ASTExpression, String> {
        let save_point = input.create_save_point();
        let mut last_err = String::new();
        for parser in RefCell::borrow(&parsers).iter() {
            input.skip_spaces_and_newlines();
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

fn parse_function_with_parser<'a>(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTExpression, String>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String> + 'a> {
    Box::new(move | input: &mut ParseInput | {
        let parameters = parse_parentheses_with_parser(interior_parser.clone())(input)?;
        input.skip_spaces_and_newlines();
        let body = parse_scope_with_parser(interior_parser.clone())(input)?;
        Ok(
            ASTExpression::ASTFunction(
                Box::new(parameters),
                Box::new(body)
            )
        )
    })
}

fn main() {
    match fs::read_to_string(TESTING_FILE_PATH) {
        Ok(contents) => {

            let parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTExpression, String>>>>> = Rc::new(RefCell::new(vec!(
                Box::new(parse_string_literal),
                Box::new(parse_integer),
                Box::new(parse_variable_ref)
            )));

            // To pass the main recursive parser around, you first prepare it and leak it here to make it static, then you dereference and re-reference it to make it immutable.
            let main_parser = &*Box::leak(try_parsers_with_list(parsers.clone()));

            // Here, I create the recursive parsers by passing the main_parser into multiple functions.
            // Wrapping main_parser in an Arc instead of a Rc adds some overhead, but should allow for multi-threaded parsing down the line. 
            let scope_parser = parse_scope_with_parser(Arc::new(main_parser));
            let parentheses_parser = parse_parentheses_with_parser(Arc::new(main_parser));
            let function_parser = parse_function_with_parser(Arc::new(main_parser));
            let assignment_parser = parse_assignment_with_parser(Arc::new(main_parser));

            // After constructing the scope_parser and passing the main parser into it, I then add the scope_parser into the main parser.
            // This allows for endless recursive parsing, but it also makes the type definitions explode in length.
            // A Rc<RefCell<Vec<Box<...>>>> doesn't exactly roll off the tongue.
            parsers.borrow_mut().push(Box::new(scope_parser));
            parsers.borrow_mut().push(Box::new(parentheses_parser));
            parsers.borrow_mut().insert(0, Box::new(function_parser));
            parsers.borrow_mut().insert(1, Box::new(assignment_parser));

            let mut input = ParseInput::new(contents);

            let mut ast_tree: Vec<ASTExpression> = Vec::new();

            // The below loop should probably go into a parse_file function somewhere.
            // If I make a Parser type for my parsing functions, maybe it could live in that type as a static method.
            loop {
                if !input.finished() {
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

            println!("{:#?}", ast_tree);
        },
        Err(err) => println!("{}", err)        
    }
}
