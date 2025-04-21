use nom::IResult;

use crate::ast::*;
use crate::lexer::Token;

pub fn parse(tokens: &[Token]) -> IResult<&[Token], Term> {
    parse_expression(tokens)
}

fn parse_expression(tokens: &[Token]) -> IResult<&[Token], Term> {
    parse_add_sub(tokens)
}

fn parse_add_sub(tokens: &[Token]) -> IResult<&[Token], Term> {
    let (mut rest, mut left) = parse_mul_div(tokens)?;

    while let Some(op_token) = rest.first() {
        match op_token {
            Token::Plus | Token::Minus => {
                let op = match op_token {
                    Token::Plus => BinaryOperator::Plus,
                    Token::Minus => BinaryOperator::Minus,
                    _ => unreachable!(),
                };

                let (r, right) = parse_mul_div(&rest[1..])?;
                left = Term::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
                rest = r;
            }
            Token::Whitespace => {
                rest = &rest[1..];
            }
            _ => break,
        }
    }

    Ok((rest, left))
}

fn parse_mul_div(tokens: &[Token]) -> IResult<&[Token], Term> {
    let (mut rest, mut left) = parse_unary(tokens)?;

    while let Some(op_token) = rest.first() {
        match op_token {
            Token::Times | Token::Div => {
                let op = match op_token {
                    Token::Times => BinaryOperator::Times,
                    Token::Div => BinaryOperator::Div,
                    _ => unreachable!(),
                };

                let (r, right) = parse_pow(&rest[1..])?;
                left = Term::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
                rest = r;
            }
            Token::Whitespace => {
                rest = &rest[1..];
            }
            _ => break,
        }
    }

    Ok((rest, left))
}

// New function to handle unary operations
fn parse_unary(tokens: &[Token]) -> IResult<&[Token], Term> {
    let mut tokens = tokens;

    // Skip whitespace
    while let Some(Token::Whitespace) = tokens.first() {
        tokens = &tokens[1..];
    }

    if tokens.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Eof,
        )));
    }

    match tokens.first() {
        Some(Token::Minus) => {
            // Handle unary minus with higher precedence than multiplication
            let (rest, operand) = parse_pow(&tokens[1..])?; // Still use parse_pow for the operand
            Ok((
                rest,
                Term::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(operand),
                },
            ))
        }
        _ => parse_pow(tokens),
    }
}

fn parse_pow(tokens: &[Token]) -> IResult<&[Token], Term> {
    let (rest, left) = parse_primary(tokens)?;

    if let Some(Token::Pow) = rest.first() {
        let (r, right) = parse_pow(&rest[1..])?;
        Ok((
            r,
            Term::BinaryOp {
                op: BinaryOperator::Pow,
                left: Box::new(left),
                right: Box::new(right),
            },
        ))
    } else {
        Ok((rest, left))
    }
}

fn parse_primary(tokens: &[Token]) -> IResult<&[Token], Term> {
    let mut tokens = tokens;

    while let Some(Token::Whitespace) = tokens.first() {
        tokens = &tokens[1..];
    }

    if tokens.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Eof,
        )));
    }

    match tokens.first() {
        Some(Token::Minus) => {
            let (rest, operand) = parse_primary(&tokens[1..])?;
            Ok((
                rest,
                Term::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(operand),
                },
            ))
        }
        Some(Token::Integer(n)) => Ok((&tokens[1..], Term::Integer(*n))),
        Some(Token::Float(f)) => Ok((&tokens[1..], Term::Float(*f))),
        Some(Token::Identifier(name)) => {
            // Check if it's a function call
            if let Some(Token::LParen) = tokens.get(1) {
                parse_function_call(tokens)
            } else {
                Ok((&tokens[1..], Term::Identifier(name.clone())))
            }
        }
        Some(Token::LParen) => {
            let (rest, expr) = parse_expression(&tokens[1..])?;
            let rest = skip_whitespace(rest);
            if let Some(Token::RParen) = rest.first() {
                Ok((&rest[1..], expr))
            } else {
                Err(nom::Err::Error(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Char,
                )))
            }
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Char,
        ))),
    }
}

fn skip_whitespace(tokens: &[Token]) -> &[Token] {
    let mut tokens = tokens;
    while let Some(Token::Whitespace) = tokens.first() {
        tokens = &tokens[1..];
    }
    tokens
}

fn parse_function_call(tokens: &[Token]) -> IResult<&[Token], Term> {
    if tokens.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Eof,
        )));
    }

    match tokens.first() {
        Some(Token::Identifier(name)) => {
            let mut rest = &tokens[1..];

            if let Some(Token::LParen) = rest.first() {
                rest = &rest[1..];

                let mut args = Vec::new();
                let mut current = rest;

                current = skip_whitespace(current);

                if let Some(Token::RParen) = current.first() {
                    return Ok((
                        &current[1..],
                        Term::FunctionCall {
                            name: name.clone(),
                            args,
                        },
                    ));
                }

                loop {
                    let (next, arg) = parse_expression(current)?;
                    args.push(arg);
                    current = skip_whitespace(next);

                    match current.first() {
                        Some(Token::Comma) => {
                            current = skip_whitespace(&current[1..]);
                        }
                        Some(Token::RParen) => {
                            return Ok((
                                &current[1..],
                                Term::FunctionCall {
                                    name: name.clone(),
                                    args,
                                },
                            ));
                        }
                        _ => {
                            return Err(nom::Err::Error(nom::error::Error::new(
                                current,
                                nom::error::ErrorKind::Char,
                            )));
                        }
                    }
                }
            } else {
                Err(nom::Err::Error(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Char,
                )))
            }
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Char,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use std::f64::consts::PI;

    #[test]
    fn test_parse_integer() {
        let tokens = tokenize("42");
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            assert_eq!(term, Term::Integer(42));
        }
    }

    #[test]
    fn test_parse_float() {
        let tokens = tokenize(format!("{}", PI).as_str());
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            assert_eq!(term, Term::Float(PI));
        }
    }

    #[test]
    fn test_parse_addition() {
        let tokens = tokenize("2 + 3");
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            assert_eq!(
                term,
                Term::BinaryOp {
                    op: BinaryOperator::Plus,
                    left: Box::new(Term::Integer(2)),
                    right: Box::new(Term::Integer(3)),
                }
            );
        }
    }

    #[test]
    fn test_parse_precedence() {
        let tokens = tokenize("2 + 3 * 4");
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            // Should parse as 2 + (3 * 4)
            assert_eq!(
                term,
                Term::BinaryOp {
                    op: BinaryOperator::Plus,
                    left: Box::new(Term::Integer(2)),
                    right: Box::new(Term::BinaryOp {
                        op: BinaryOperator::Times,
                        left: Box::new(Term::Integer(3)),
                        right: Box::new(Term::Integer(4)),
                    }),
                }
            );
        }
    }

    #[test]
    fn test_parse_unary_precedence() {
        let tokens = tokenize("-2^2");
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            // Should parse as -(2^2)
            assert_eq!(
                term,
                Term::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Term::BinaryOp {
                        op: BinaryOperator::Pow,
                        left: Box::new(Term::Integer(2)),
                        right: Box::new(Term::Integer(2)),
                    }),
                }
            );
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let tokens = tokenize("(1 + 2) * 3");
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            assert_eq!(
                term,
                Term::BinaryOp {
                    op: BinaryOperator::Times,
                    left: Box::new(Term::BinaryOp {
                        op: BinaryOperator::Plus,
                        left: Box::new(Term::Integer(1)),
                        right: Box::new(Term::Integer(2)),
                    }),
                    right: Box::new(Term::Integer(3)),
                }
            );
        }
    }

    #[test]
    fn test_parse_function_call() {
        let tokens = tokenize("ID(42)");
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            assert_eq!(
                term,
                Term::FunctionCall {
                    name: "ID".to_string(),
                    args: vec![Term::Integer(42)],
                }
            );
        }
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let tokens = tokenize("ID(ID(42))");
        let result = parse(&tokens);
        assert!(result.is_ok());
        if let Ok((_, term)) = result {
            assert_eq!(
                term,
                Term::FunctionCall {
                    name: "ID".to_string(),
                    args: vec![Term::FunctionCall {
                        name: "ID".to_string(),
                        args: vec![Term::Integer(42)],
                    }],
                }
            );
        }
    }
}
