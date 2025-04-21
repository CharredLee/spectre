#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    Integer(i64),
    Float(f64),
    Plus,
    Minus,
    Times,
    Div,
    Pow,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    Comma,
    Whitespace,
    Unknown(char),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '[' => tokens.push(Token::LBracket),
            ']' => tokens.push(Token::RBracket),
            '{' => tokens.push(Token::LCurly),
            '}' => tokens.push(Token::RCurly),
            ',' => tokens.push(Token::Comma),
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Times),
            '/' => tokens.push(Token::Div),
            '^' => tokens.push(Token::Pow),
            '0'..='9' | '.' => {
                let mut float = c == '.';
                let mut num = c.to_string();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() {
                        num.push(next_c);
                        chars.next();
                    } else if next_c == '.' {
                        float = true;
                        num.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if float {
                    tokens.push(Token::Float(num.parse().unwrap()));
                } else {
                    tokens.push(Token::Integer(num.parse().unwrap()));
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = c.to_string();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        ident.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(ident));
            }
            ' ' | '\t' | '\n' | '\r' => tokens.push(Token::Whitespace),
            _ => tokens.push(Token::Unknown(c)),
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_empty() {
        let result = tokenize("");
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_tokenize_identifier() {
        let result = tokenize("f");
        assert_eq!(result, vec![Token::Identifier("f".to_string())]);
    }

    #[test]
    fn test_tokenize_integer() {
        let result1 = tokenize("123");
        let result2 = tokenize("-13");
        assert_eq!(result1, vec![Token::Integer(123)]);
        assert_eq!(result2, vec![Token::Minus, Token::Integer(13)]);
    }

    #[test]
    fn test_tokenize_float() {
        let result1 = tokenize(".123");
        let result2 = tokenize("-1.3");
        assert_eq!(result1, vec![Token::Float(0.123)]);
        assert_eq!(result2, vec![Token::Minus, Token::Float(1.3)]);
    }

    #[test]
    fn test_tokenize_function_call() {
        let result = tokenize("f({5},[3])");
        assert_eq!(
            result,
            vec![
                Token::Identifier("f".to_string()),
                Token::LParen,
                Token::LCurly,
                Token::Integer(5),
                Token::RCurly,
                Token::Comma,
                Token::LBracket,
                Token::Integer(3),
                Token::RBracket,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_tokenize_with_whitespace() {
        let result = tokenize("f ( 5 , 3 )");
        assert_eq!(
            result,
            vec![
                Token::Identifier("f".to_string()),
                Token::Whitespace,
                Token::LParen,
                Token::Whitespace,
                Token::Integer(5),
                Token::Whitespace,
                Token::Comma,
                Token::Whitespace,
                Token::Integer(3),
                Token::Whitespace,
                Token::RParen,
            ]
        );
    }
}

