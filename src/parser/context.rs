use crate::ast::FunctionCall;
use nom::{Parser, error::Error as NomError};
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub function_call_format: FunctionCallFormat,
    // pub function_def_format: FunctionDefFormat,
    // pub if_else_format: IfElseFormat,
    // pub string_format: StringFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallFormat {
    pattern: String,       // e.g. "NAME(ARGS)"
    arg_separator: String, // e.g. ","
}

impl FunctionCallFormat {
    pub fn new(pattern: String, arg_separator: String) -> Self {
        Self {
            pattern,
            arg_separator,
        }
    }

    pub fn pattern(&self) -> &String {
        &self.pattern
    }

    pub fn arg_separator(&self) -> &String {
        &self.arg_separator
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new(FunctionCallFormat::new(
            "(NAME ARGS)".to_string(),
            " ".to_string(),
        ))
    }
}

impl Context {
    pub fn new(function_call_format: FunctionCallFormat) -> Self {
        Context {
            function_call_format,
        }
    }

    pub fn generate_function_call_regex(&self) -> Result<Regex, String> {
        let pattern = self.function_call_format.pattern.clone();
        let arg_separator = self.function_call_format.arg_separator.clone();
        let name_count = pattern.matches("NAME").count();
        let args_count = pattern.matches("ARGS").count();

        if name_count != 1 {
            return Err(format!(
                "Pattern must contain exactly one 'NAME', found {}",
                name_count
            ));
        }

        if args_count != 1 {
            return Err(format!(
                "Pattern must contain exactly one 'ARGS', found {}",
                args_count
            ));
        }

        let escaped_separator = regex::escape(&arg_separator);
        let regex_string = pattern
            .replace("NAME", r"([a-zA-Z_][a-zA-Z0-9_]*)")
            .replace("ARGS", r"(.*?)");

        Regex::new(&regex_string).map_err(|e| format!("Invalid regex: {}", e))
    }

    pub fn update_function_call_format(
        &mut self,
        pattern: String,
        arg_separator: String,
    ) -> Result<(), String> {
        self.function_call_format = FunctionCallFormat::new(pattern, arg_separator);
        Ok(())
    }
}
