use crate::ast_parsers::*;
use nom::branch::alt;
use nom::character::complete::multispace0;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::IResult;

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    VariableRef(String),
    Integer(i64),
    Float(f64),
    String(String),
    Assignment(String, Box<AST>),
    Scope(Vec<AST>),
    Parentheses(Vec<AST>),
    Function(String, Box<AST>),
}

impl AST {
    pub fn parse(input: &str) -> IResult<&str, AST> {
        alt((
            parse_float,
            parse_integer,
            parse_assignment,
            parse_function,
            parse_scope,
            parse_parentheses,
            parse_string_literal,
            parse_variable_ref,
        ))(input)
    }

    pub fn parse_file(input: &str) -> IResult<&str, Vec<AST>> {
        many0(delimited(multispace0, Self::parse, multispace0))(input)
    }
}
