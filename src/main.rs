mod ast;
mod ast_parsers;
mod parse_identifiers;
mod parse_numbers;
mod parse_strings;

use std::fs;

use crate::ast::AST;

const TESTING_FILE_PATH: &str = "test.txt";

fn main() {
    match fs::read_to_string(TESTING_FILE_PATH) {
        Ok(contents) => {
            let ast_tree = AST::parse_file(&contents);

            println!("{:#?}", ast_tree);
        }
        Err(err) => println!("{}", err),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::AST;

    #[test]
    fn parse_integers() {
        assert_eq!(AST::parse("-99"), Ok(("", AST::Integer(-99))));
        assert_eq!(AST::parse("250"), Ok(("", AST::Integer(250))));
        assert_eq!(AST::parse("1000"), Ok(("", AST::Integer(1_000))));
    }

    #[test]
    fn parse_floats() {
        assert_eq!(AST::parse("-0.25"), Ok(("", AST::Float(-0.25))));
        assert_eq!(AST::parse("-.75"), Ok(("", AST::Float(-0.75))));
        assert_eq!(AST::parse("99.99"), Ok(("", AST::Float(99.99))));
    }

    #[test]
    fn parse_strings() {
        assert_eq!(
            AST::parse("\"a string\""),
            Ok(("", AST::String(String::from("a string"))))
        );
        assert_eq!(
            AST::parse("\"line1 \r\nline2\nline3\""),
            Ok(("", AST::String(String::from("line1 \r\nline2\nline3"))))
        );
        assert_eq!(
            AST::parse("\"😉\""),
            Ok(("", AST::String(String::from("😉"))))
        );
    }

    #[test]
    fn parse_assignment() {
        assert_eq!(
            AST::parse("let x = 0"),
            Ok((
                "",
                AST::Assignment(String::from("x"), Box::new(AST::Integer(0)))
            ))
        );
        assert_eq!(
            AST::parse("let x = -999"),
            Ok((
                "",
                AST::Assignment(String::from("x"), Box::new(AST::Integer(-999)))
            ))
        );
    }

    #[test]
    fn parse_parentheses() {
        assert_eq!(
            AST::parse("(1 2 3.0000001)"),
            Ok((
                "",
                AST::Parentheses(vec![
                    AST::Integer(1),
                    AST::Integer(2),
                    AST::Float(3.0000001),
                ])
            ))
        );

        assert_eq!(
            AST::parse("(0 (1 (2 (3))))"),
            Ok((
                "",
                AST::Parentheses(vec![
                    AST::Integer(0),
                    AST::Parentheses(vec![
                        AST::Integer(1),
                        AST::Parentheses(vec![
                            AST::Integer(2),
                            AST::Parentheses(vec![AST::Integer(3)]),
                        ]),
                    ]),
                ])
            ))
        );
    }

    #[test]
    fn parse_scopes() {
        assert_eq!(
            AST::parse("{1 2 3.0000001}"),
            Ok((
                "",
                AST::Scope(vec![
                    AST::Integer(1),
                    AST::Integer(2),
                    AST::Float(3.0000001),
                ])
            ))
        );

        assert_eq!(
            AST::parse("{0 {1 {2 {3}}}}"),
            Ok((
                "",
                AST::Scope(vec![
                    AST::Integer(0),
                    AST::Scope(vec![
                        AST::Integer(1),
                        AST::Scope(vec![AST::Integer(2), AST::Scope(vec![AST::Integer(3)]),]),
                    ]),
                ])
            ))
        );

        assert_eq!(
            AST::parse(
                r#"{
                
                }"#
            ),
            Ok((
                "",
                AST::Scope(vec![])
            ))
        );
    }

    #[test]
    fn parse_functions() {
        assert_eq!(
            AST::parse("|x| x"),
            Ok((
                "",
                AST::Function("x".to_owned(), Box::new(AST::VariableRef("x".to_owned())))
            ))
        );

        assert_eq!(
            AST::parse("|x y z| (x y z)"),
            Ok((
                "",
                AST::Function(
                    "x".to_owned(),
                    Box::new(AST::Function(
                        "y".to_owned(),
                        Box::new(AST::Function(
                            "z".to_owned(),
                            Box::new(AST::Parentheses(vec![
                                AST::VariableRef(String::from("x")),
                                AST::VariableRef(String::from("y")),
                                AST::VariableRef(String::from("z")),
                            ]))
                        ))
                    ))
                )
            ))
        );

        assert_eq!(
            AST::parse("let adder = |add_amount x| (+ add_amount x)"),
            AST::parse("let adder = |add_amount| |x| (+ add_amount x)")
        );
    }
}
