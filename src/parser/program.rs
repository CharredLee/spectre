use crate::ast::*;
use crate::parser::context::*;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{multispace0, multispace1},
    combinator::{map, opt, recognize},
    error::Error,
    multi::separated_list0,
    sequence::{delimited, pair, preceded, terminated},
};
use regex::Regex;

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((
        map(
            delimited(tag("\""), take_while(|c: char| c != '"'), tag("\"")),
            |s: &str| Literal::String(s.to_string()),
        ),
        // Boolean literal
        map(tag("true"), |_| Literal::Boolean(true)),
        map(tag("false"), |_| Literal::Boolean(false)),
        // Integer literal (simplified)
        map(take_while1(|c: char| c.is_ascii_digit()), |s: &str| {
            Literal::Integer(s.parse().unwrap())
        }),
    ))
    .parse(input)
}

fn parse_function_call<'a>(context: &'a Context, input: &'a str) -> IResult<&'a str, FunctionCall> {
    let (input, _) = multispace0(input)?;

    let regex = context.generate_function_call_regex().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::RegexpMatch,
        ))
    })?;

    let end_pos = input
        .find(|c: char| ['\n', '\r'].contains(&c))
        .unwrap_or(input.len());
    let line = &input[..end_pos];

    if let Some(captures) = regex.captures(line.trim()) {
        let name = captures.get(1).unwrap().as_str().to_string();
        let args_str = captures.get(2).unwrap().as_str();

        let mut failed = false;
        let args = if args_str.trim().is_empty() {
            Vec::new()
        } else {
            args_str
                .split(context.function_call_format.arg_separator())
                .map(|arg| {
                    let trimmed = arg.trim();
                    if let Ok((_, literal)) = parse_literal(trimmed) {
                        Ast::Literal(literal)
                    } else {
                        failed = true;
                        Ast::Literal(Literal::String(trimmed.to_string()))
                    }
                })
                .collect()
        };
        if failed {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Fail,
            )))
        } else {
            Ok((&input[end_pos..], FunctionCall { name, args }))
        }
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::RegexpMatch,
        )))
    }
}

pub fn parse_expression<'a>(
    context: &'a Context,
    input: &'a str,
) -> IResult<&'a str, (Ast, Context)> {
    todo!()
}

pub fn parse_program(input: &str) -> Result<Vec<Ast>, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::context::Context;

    #[test]
    fn test_parse_expression() {
        let function_call_format =
            FunctionCallFormat::new("NAME(ARGS)".to_string(), " ".to_string());
        let mut context = Context::new(function_call_format);

        let input = "foo(bar baz)";
        let result = parse_expression(&mut context, input);

        assert!(result.is_ok());
        let (rest, (ast, new_context)) = result.unwrap();
        assert_eq!(rest, "");

        match ast {
            Ast::FunctionCall(call) => {
                assert_eq!(call.name, "foo");
                assert_eq!(call.args.len(), 2);
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_parse_program() {
        let input = r#"{"NAME(ARGS)" " "}
        foo(bar baz)
        SPEC(function_call_format "NAME:ARGS" ",")
        bar:qux,quux"#;

        let result = parse_program(input);
        assert!(result.is_ok());

        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 3);

        // First node: foo(bar baz) with original syntax
        match &ast_nodes[0] {
            Ast::FunctionCall(call) => {
                assert_eq!(call.name, "foo");
                assert_eq!(call.args.len(), 2);
            }
            _ => panic!("Expected FunctionCall"),
        }

        // Second node: SPEC function call
        match &ast_nodes[1] {
            Ast::FunctionCall(call) if call.name == "SPEC" => {
                assert_eq!(call.args.len(), 3);
            }
            _ => panic!("Expected SPEC call"),
        }

        // Third node: bar:qux,quux with updated syntax
        match &ast_nodes[2] {
            Ast::FunctionCall(call) => {
                assert_eq!(call.name, "bar");
                assert_eq!(call.args.len(), 2);
            }
            _ => panic!("Expected FunctionCall with updated syntax"),
        }
    }

    #[test]
    fn test_parse_invalid_program() {
        let input = r#"{"NAME(ARGS)" ",\s*"}
        foo(bar, baz)
        SPEC(function_call_format, "NAME:ARGS", " ")
        foo(bar, baz)
        "#;

        let result = parse_program(input);
        assert!(result.is_err());
    }
}
