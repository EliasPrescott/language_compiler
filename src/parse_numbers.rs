use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::opt;
use nom::sequence::{preceded, tuple};
use nom::{
    character::complete::one_of,
    combinator::recognize,
    multi::{many0, many1},
    sequence::terminated,
    IResult,
};

// Stolen from https://docs.rs/nom/latest/nom/recipes/index.html#decimal
pub fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}

pub fn integer(input: &str) -> IResult<&str, &str> {
    recognize(tuple((opt(char('-')), decimal)))(input)
}

// Stolen from https://docs.rs/nom/latest/nom/recipes/index.html#floating-point-numbers
pub fn float(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        opt(char('-')),
        alt((
            // Case one: .42
            recognize(tuple((
                char('.'),
                decimal,
                opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
            ))), // Case two: 42e42 and 42.42e42
            recognize(tuple((
                decimal,
                opt(preceded(char('.'), decimal)),
                one_of("eE"),
                opt(one_of("+-")),
                decimal,
            ))), // Case three: 42. and 42.42
            recognize(tuple((decimal, char('.'), opt(decimal)))),
        )),
    )))(input)
}
