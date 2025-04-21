mod ast;
mod interpreter;
mod lexer;
mod parser;
mod repl;

use std::env;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Run file mode
        let filename = &args[1];
        let mut file = fs::File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut interpreter = interpreter::Interpreter::new();
        let tokens = lexer::tokenize(&contents);
        let tokens: Vec<lexer::Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t, lexer::Token::Whitespace))
            .collect();

        match parser::parse(&tokens) {
            Ok((_, ast)) => match interpreter.interpret(ast) {
                Ok(value) => match value {
                    interpreter::Value::Integer(n) => println!("{}", n),
                    interpreter::Value::Float(f) => println!("{}", f),
                    interpreter::Value::Unit => {}
                    _ => println!("{:?}", value),
                },
                Err(err) => eprintln!("Error: {}", err),
            },
            Err(err) => eprintln!("Parse error: {:?}", err),
        }
    } else {
        // REPL mode
        println!("welcome to REPL!");

        if let Err(e) = repl::start() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

