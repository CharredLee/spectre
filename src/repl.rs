use crate::interpreter::{Interpreter, Value};
use crate::lexer::*;
use crate::parser::parse;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use std::collections::VecDeque;
use std::io::{self, Write};

const HISTORY_SIZE: usize = 100;

pub fn start() -> Result<(), io::Error> {
    let mut interpreter = Interpreter::new();
    let mut history: VecDeque<String> = VecDeque::with_capacity(HISTORY_SIZE);
    let mut history_index: Option<usize> = None;
    let mut current_line = String::new();
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;

    loop {
        execute!(stdout, cursor::MoveToColumn(0), Print(">> "))?;

        match read_line(&mut current_line, &mut history, &mut history_index)? {
            LineReadAction::Line => {
                execute!(stdout, Print("\r\n"))?;

                if !current_line.trim().is_empty() {
                    if history.len() >= HISTORY_SIZE {
                        history.pop_front();
                    }
                    history.push_back(current_line.clone());
                }

                let tokens: Vec<Token> = tokenize(&current_line)
                    .into_iter()
                    .filter(|token| !matches!(token, Token::Whitespace))
                    .collect();
                match parse(&tokens) {
                    Ok((_, ast)) => match interpreter.interpret(ast) {
                        Ok(value) => match value {
                            Value::Integer(n) => {
                                execute!(stdout, Print(n), Print("\r\n"))?;
                            }
                            Value::Float(f) => {
                                execute!(stdout, Print(f), Print("\r\n"))?;
                            }
                            Value::Function { .. } => {
                                execute!(stdout, Print("Function created"), Print("\r\n"))?;
                            }
                            Value::Builtin(name) => {
                                execute!(
                                    stdout,
                                    Print(format!("Builtin: {}", name)),
                                    Print("\r\n")
                                )?;
                            }
                            Value::Unit => {}
                        },
                        Err(err) => {
                            execute!(stdout, Print(format!("Error: {}", err)), Print("\r\n"))?;
                        }
                    },
                    Err(err) => {
                        execute!(
                            stdout,
                            Print(format!("Parse error: {:?}", err)),
                            Print("\r\n")
                        )?;
                    }
                }
                current_line.clear();
                history_index = None;
            }
            LineReadAction::Exit => {
                execute!(stdout, Print("\r\n"), Print("Exiting..."), Print("\r\n"))?;
                terminal::disable_raw_mode()?;
                std::io::stdout().flush()?;
                return Ok(());
            }
        }
    }
}

enum LineReadAction {
    Line,
    Exit,
}

fn read_line(
    current_line: &mut String,
    history: &mut VecDeque<String>,
    history_index: &mut Option<usize>,
) -> Result<LineReadAction, io::Error> {
    let mut stdout = io::stdout();

    loop {
        if let Event::Key(key_event) = event::read()? {
            match key_event {
                KeyEvent {
                    code: KeyCode::Char('c') | KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    return Ok(LineReadAction::Exit);
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => {
                    return Ok(LineReadAction::Line);
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    if !current_line.is_empty() {
                        current_line.pop();
                        // Clear line and reprint
                        execute!(
                            stdout,
                            cursor::MoveToColumn(0),
                            Clear(ClearType::CurrentLine),
                            Print(">> "),
                            Print(current_line.as_str())
                        )?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Up, ..
                } => {
                    if !history.is_empty() {
                        let new_index = match history_index {
                            None => history.len() - 1,
                            Some(i) if *i > 0 => *i - 1,
                            Some(_) => 0,
                        };

                        if let Some(hist_cmd) = history.get(new_index) {
                            *history_index = Some(new_index);
                            *current_line = hist_cmd.clone();

                            // Clear line and reprint
                            execute!(
                                stdout,
                                cursor::MoveToColumn(0),
                                Clear(ClearType::CurrentLine),
                                Print(">> "),
                                Print(current_line.as_str())
                            )?;
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => {
                    match history_index {
                        Some(i) if *i + 1 < history.len() => {
                            let new_index = *i + 1;
                            if let Some(hist_cmd) = history.get(new_index) {
                                *history_index = Some(new_index);
                                *current_line = hist_cmd.clone();

                                // Clear line and reprint
                                execute!(
                                    stdout,
                                    cursor::MoveToColumn(0),
                                    Clear(ClearType::CurrentLine),
                                    Print(">> "),
                                    Print(current_line.as_str())
                                )?;
                            }
                        }
                        Some(_) => {
                            // At the end of history
                            *history_index = None;
                            current_line.clear();

                            // Clear line and reprint
                            execute!(
                                stdout,
                                cursor::MoveToColumn(0),
                                Clear(ClearType::CurrentLine),
                                Print(">> ")
                            )?;
                        }
                        None => {} // Already at the current input
                    }
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => {
                    current_line.push(c);
                    execute!(stdout, Print(c))?;
                }
                _ => {}
            }
        }
    }
}
