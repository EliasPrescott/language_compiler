use std::{rc::Rc, cell::RefCell, sync::Arc};
use crate::parsing::*;
use crate::AST::*;

pub fn parse_integer(input: &mut ParseInput) -> Result<ASTNode, String> {
    // The pop_next_char_numerical and other similar methods only mutate the ParseInput if the next char matches the predicate.
    // This fact will need to be explicit in the documentation for these methods.
    let first_char = input.pop_next_char_numerical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_numerical() {
        output += &next_char.to_string();
    }
    Ok(ASTNode::new(ASTExpression::ASTInteger(str::parse::<i64>(&output).map_err(|err| err.to_string())?), first_char.line, first_char.column))
}

pub fn parse_string_literal(input: &mut ParseInput) -> Result<ASTNode, String> {
    // The question mark operator at the end of these parser functions will return early if that expression is a Result::Err case.
    // This allows for extremely concise and convenient error-handling.
    let first_char = input.pop_char('"')?;
    let output = input.pop_until_char('"');
    input.skip_char('"')?;
    Ok(ASTNode::new(ASTExpression::ASTString(output), first_char.line, first_char.column))
}

pub fn parse_name(input: &mut ParseInput) -> Result<String, String> {
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
pub fn parse_variable_ref(input: &mut ParseInput) -> Result<ASTNode, String> {
    let accepted_nonpreceding_symbols = vec!('_', '-');
    let first_char = input.pop_next_char_alphabetical()?;
    let mut output = first_char.to_string();
    while let Ok(next_char) = input.pop_next_char_alphabetical_or_in_group(&accepted_nonpreceding_symbols) {
        output += &next_char.to_string();
    }
    Ok(ASTNode::new(ASTExpression::ASTVariableRef(output), first_char.line, first_char.column))
}

// Instead of parsing input directly, this function takes in an interior_parser, and then builds a new parser that will continually run that interior_parser within a braced scope.
pub fn parse_scope_with_parser(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>> {
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

pub fn parse_parentheses_with_parser(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>> {
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

pub fn parse_parentheses_with_parsers<'a>(parsers: Box<Vec<&'a (dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a)>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
    Box::new(move | input: &mut ParseInput | {
        let mut output = vec!();
        let first_char = input.pop_char('(')?;
        while input.skip_char(')').is_err() {
            output.push(Box::new(try_parsers(input, parsers.to_vec())?));
        }
        Ok(ASTNode::new(ASTExpression::ASTParentheses(output), first_char.line, first_char.column))
    })
}

pub fn parse_assignment_with_parser<'a>(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
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
pub fn try_parsers(input: &mut ParseInput, parsers: Vec<&dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Result<ASTNode, String> {
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
pub fn try_parsers_with_list<'a>(parsers: Rc<RefCell<Vec<Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String>>>>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
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

pub fn parse_function_with_parser<'a>(interior_parser: Arc<dyn Fn(&mut ParseInput) -> Result<ASTNode, String>>) -> Box<dyn for<'r> Fn(&'r mut ParseInput) -> Result<ASTNode, String> + 'a> {
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
