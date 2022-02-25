mod parsing;

use std::{fs, rc::Rc, cell::RefCell, sync::Arc};
use parsing::*;

const TESTING_FILE_PATH: &str = "test.txt";

#[derive(Debug, Clone, Copy)]
struct ASTLocation {
    line: u32,
    column: u32,
}

#[derive(Debug, Clone)]
enum ASTExpression {
    ASTVariableRef(String),
    ASTInteger(i64),
    ASTString(String),
    ASTAssignment(String, Box<ASTNode>),
    ASTInitialization(String, Box<ASTNode>),
    ASTScope(Vec<Box<ASTNode>>),
    ASTParentheses(Vec<Box<ASTNode>>),
    ASTFunction(Box<ASTNode>, Box<ASTNode>),
    ASTUnit,
}

#[derive(Debug, Clone)]
struct ASTNode {
    expression: ASTExpression,
    location: ASTLocation,
}

impl ASTNode {
    fn new(expression: ASTExpression, line: u32, column: u32) -> Self {
        ASTNode {
            expression,
            location: ASTLocation {
                line,
                column
            },
        }
    }
}

fn parse_integer(input: &mut ParseInput) -> Result<ASTNode, String> {
    // The pop_next_char_numerical and other similar methods only mutate the ParseInput if the next char matches the predicate.
    // This fact will need to be explicit in the documentation for these methods.
    let first_char = input.pop_next_char_numerical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_numerical() {
        output += &next_char.to_string();
    }
    Ok(ASTNode::new(ASTExpression::ASTInteger(str::parse::<i64>(&output).map_err(|err| err.to_string())?), first_char.line, first_char.column))
}

fn parse_string_literal(input: &mut ParseInput) -> Result<ASTNode, String> {
    // The question mark operator at the end of these parser functions will return early if that expression is a Result::Err case.
    // This allows for extremely concise and convenient error-handling.
    let first_char = input.pop_char('"')?;
    let output = input.pop_until_char('"');
    input.skip_char('"')?;
    Ok(ASTNode::new(ASTExpression::ASTString(output), first_char.line, first_char.column))
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
fn parse_variable_ref(input: &mut ParseInput) -> Result<ASTNode, String> {
    let accepted_nonpreceding_symbols = vec!('_', '-');
    let first_char = input.pop_next_char_alphabetical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_alphabetical_or_in_group(&accepted_nonpreceding_symbols) {
        output += &next_char.to_string();
    }
    Ok(ASTNode::new(ASTExpression::ASTVariableRef(output), first_char.line, first_char.column))
}

// Instead of parsing input directly, this function takes in an interior_parser, and then builds a new parser that will continually run that interior_parser within a braced scope.
fn parse_scope_with_parser(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>> {
    Box::new(move | input: &mut ParseInput | {
        let mut output = vec!();
        let start_char = input.pop_char('{')?;
        loop {
            input.skip_spaces_and_newlines();
            match interior_parser(input) {
                Ok(x) => {
                    output.push(Box::new(x));
                },
                Err(e) => {
                    if let Ok(()) = input.skip_char('}') {
                        return Ok(ASTNode::new(ASTExpression::ASTScope(output), start_char.line, start_char.column));
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    })
}

fn parse_parentheses_with_parser(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>> {
    Box::new(move | input: &mut ParseInput | {
        let mut output = vec!();
        let first_char = input.pop_char('(')?;
        loop {
            input.skip_spaces_and_newlines();
            match interior_parser(input) {
                Ok(x) => {
                    output.push(Box::new(x));
                },
                Err(e) => {
                    if let Ok(()) = input.skip_char(')') {
                        return Ok(ASTNode::new(ASTExpression::ASTParentheses(output), first_char.line, first_char.column));
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    })
}

fn parse_parentheses_with_parsers<'a>(parsers: Box<Vec<&'a (dyn for<'r> Fn(&'r mut parsing::ParseInput) -> Result<ASTNode, String> + 'a)>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
    Box::new(move | input: &mut ParseInput | {
        let mut output = vec!();
        let first_char = input.pop_char('(')?;
        while input.skip_char(')').is_err() {
            output.push(Box::new(try_parsers(input, parsers.to_vec())?));
        }
        Ok(ASTNode::new(ASTExpression::ASTParentheses(output), first_char.line, first_char.column))
    })
}

fn parse_assignment_with_parser<'a>(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
    Box::new(move | input: &mut ParseInput | {
        let mut initialization = false;
        let mut first_char =
            if input.match_word("let") {
                initialization = true;
                input.get_next_char()
            } else {
                None
            };
        println!("{}", initialization);
        let _ = input.skip_string("let");
        input.skip_spaces_and_newlines();
        if !initialization {
            let new_first_char = input.get_next_char_result()?;
            first_char = Some(new_first_char)
        }
        let variable_name = parse_name(input)?;
        input.skip_spaces_and_newlines();
        input.skip_char('=')?;
        input.skip_spaces_and_newlines();
        let variable_value = interior_parser(input)?;

        if initialization {
            Ok(ASTNode::new(ASTExpression::ASTInitialization(variable_name, Box::new(variable_value)), first_char.unwrap().line, first_char.unwrap().column))
        } else {
            Ok(ASTNode::new(ASTExpression::ASTAssignment(variable_name, Box::new(variable_value)), first_char.unwrap().line, first_char.unwrap().column))
        }
    })
}

// A simpler version of try_parsers that does not support adding in more parsers later for recursion purposes
fn try_parsers(input: &mut ParseInput, parsers: Vec<&dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Result<ASTNode, String> {
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
fn try_parsers_with_list<'a>(parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String>>>>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
    Box::new(move | input: &mut ParseInput | -> Result<ASTNode, String> {
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

fn parse_function_with_parser<'a>(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
    Box::new(move | input: &mut ParseInput | {
        // Grabbing the first char to use for location data.
        let first_char = input.get_next_char_result()?;
        let parameters = parse_parentheses_with_parser(interior_parser.clone())(input)?;
        input.skip_spaces_and_newlines();
        let body = parse_scope_with_parser(interior_parser.clone())(input)?;
        Ok(
            ASTNode::new(
                ASTExpression::ASTFunction(
                    Box::new(parameters),
                    Box::new(body)
                ),
                first_char.line,
                first_char.column
            )
        )
    })
}

fn main() {
    match fs::read_to_string(TESTING_FILE_PATH) {
        Ok(contents) => {

            let parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String>>>>> = Rc::new(RefCell::new(vec!(
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

            let mut ast_tree: Vec<ASTNode> = Vec::new();

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
