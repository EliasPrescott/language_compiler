use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::recognize,
    multi::many0_count,
    sequence::pair,
    IResult,
};

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"), tag("+"), tag("-"), tag("$"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("+"), tag("-"), tag("$")))),
    ))(input)
}
