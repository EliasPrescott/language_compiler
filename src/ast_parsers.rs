use nom::bytes::streaming::tag;
use nom::character::complete::char;
use nom::character::complete::multispace0;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::multi::many0;
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::sequence::tuple;
use nom::IResult;

use crate::parse_identifiers::identifier;
use crate::parse_numbers;
use crate::parse_strings::parse_string;
use crate::ast::*;

type ParseResult<'a> = IResult<&'a str, AST>;

pub fn parse_integer(input: &str) -> ParseResult {
    map(parse_numbers::integer, |x| {
        AST::Integer(str::parse::<i64>(x).unwrap())
    })(input)
}

pub fn parse_float(input: &str) -> ParseResult {
    map(parse_numbers::float, |x| {
        AST::Float(str::parse::<f64>(x).unwrap())
    })(input)
}

pub fn parse_string_literal(input: &str) -> ParseResult {
    map(parse_string, AST::String)(input)
}

// Since this pattern is so permissive, it is important to put it after other, stricter patterns.
// It could be good to make a list of language keywords (e.g. 'let') and forbid them from being parsed as variable names.
pub fn parse_variable_ref(input: &str) -> ParseResult {
    map(identifier, |x| AST::VariableRef(x.to_owned()))(input)
}

pub fn parse_scope(input: &str) -> ParseResult {
    map(
        delimited(
            tuple((char('{'), multispace0)),
            many0(delimited(multispace0, AST::parse, multispace0)),
            tuple((char('}'), multispace0)),
        ),
        AST::Scope,
    )(input)
}

pub fn parse_parentheses(input: &str) -> ParseResult {
    map(
        delimited(
            tuple((char('('), multispace0)),
            many0(delimited(multispace0, AST::parse, multispace0)),
            tuple((char(')'), multispace0)),
        ),
        AST::Parentheses,
    )(input)
}

pub fn parse_assignment(input: &str) -> ParseResult {
    map(
        tuple((
            tag("let"),
            space1,
            identifier,
            space1,
            char('='),
            space1,
            AST::parse,
        )),
        |(_, _, ident, _, _, _, ast)| AST::Assignment(ident.to_owned(), Box::new(ast)),
    )(input)
}

pub fn parse_function(input: &str) -> ParseResult {
    let (rem, _) = char('|')(input)?;
    let (rem, args) = separated_list1(space1, identifier)(rem)?;
    let (rem, _) = char('|')(rem)?;
    let (rem, _) = space1(rem)?;
    let (rem, ast) = AST::parse(rem)?;

    let curried_func = args.into_iter().rev().fold(ast, |prev_body, next_ident| {
        AST::Function(next_ident.to_owned(), Box::new(prev_body))
    });

    Ok((rem, curried_func))
}
