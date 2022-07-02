use std::{rc::Rc, cell::RefCell, sync::Arc};
use crate::parsing::*;
use crate::parsers::*;

#[derive(Debug, Clone, Copy)]
pub struct ASTLocation {
    line: u32,
    column: u32,
}

#[derive(Debug, Clone)]
pub enum ASTExpression {
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
pub struct ASTNode {
    expression: ASTExpression,
    location: ASTLocation,
}

impl ASTNode {
    pub fn new(expression: ASTExpression, line: u32, column: u32) -> Self {
        ASTNode {
            expression,
            location: ASTLocation {
                line,
                column
            },
        }
    }
}

pub fn parse_ast_text(text: String) -> Vec<ASTNode> {
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

    let mut input = ParseInput::new(text);

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

    ast_tree
}