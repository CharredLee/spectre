pub mod ast;
mod error;
pub mod parser;

use std::env;
use std::fs;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return Err("Missing file argument".to_string());
    }

    let file_path = &args[1];
    let input = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let ast = parser::program::parse_program(&input)?;
    println!("Parsed AST: {:#?}", ast);

    Ok(())
}
