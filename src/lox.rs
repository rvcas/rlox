use std::{
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    interpreter::{Interpreter, InterpreterError},
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    token::Token,
    token_type::TokenType,
};

static HAD_ERROR: AtomicBool = AtomicBool::new(false);
static HAD_RUNTIME_ERROR: AtomicBool = AtomicBool::new(false);

pub fn run_file(path_name: &str) {
    let file_path = Path::new(path_name);

    let file_res = File::open(file_path);

    match file_res {
        Ok(mut src_file) => {
            let mut src = String::new();

            let read_res = src_file.read_to_string(&mut src);

            match read_res {
                Ok(_) => {
                    let mut interpreter = Interpreter::new();

                    run(&src, &mut interpreter);

                    if had_error() {
                        std::process::exit(65);
                    }

                    if had_runtime_error() {
                        std::process::exit(70);
                    }
                }
                Err(_) => println!("error: could not read {}", path_name),
            }
        }
        Err(_) => println!("error: could not open {}", path_name),
    };
}

pub fn run_prompt() {
    let mut input = String::new();

    let mut interpreter = Interpreter::new();

    loop {
        print!("> ");

        let _ = stdout().flush();

        match stdin().read_line(&mut input) {
            Ok(_) => {
                if let Some('\n') = input.chars().next_back() {
                    input.pop();
                }

                if let Some('\r') = input.chars().next_back() {
                    input.pop();
                }

                run(&input, &mut interpreter);

                set_had_error(false);
                set_had_runtime_error(false);
            }
            Err(_) => {
                println!("error: bad input");
            }
        }

        input.clear();
    }
}

fn run(src: &str, interpreter: &mut Interpreter) {
    let mut scanner = Scanner::new(src);

    let tokens = scanner.scan_tokens();

    if had_error() {
        return;
    }

    let mut parser = Parser::new(tokens.clone());

    let statements = parser.parse();

    if had_error() {
        return;
    }

    let mut resolver = Resolver::new(interpreter);

    resolver.resolve(&statements);

    if had_error() {
        return;
    }

    interpreter.interpret(&statements);
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, where_: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, where_, message);

    set_had_error(true);
}

pub fn parse_error(token: &Token, message: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, " at end", message)
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme), message)
    }
}

pub fn runtime_error(err: InterpreterError) {
    if let InterpreterError::RuntimeError(err) = err {
        if let Some(token) = err.token {
            println!("{}\n[line {}]", err.message, token.line);
        } else {
            println!("{}", err.message);
        }

        set_had_runtime_error(true);
    }
}

fn had_error() -> bool {
    HAD_ERROR.load(Ordering::Relaxed)
}

fn set_had_error(b: bool) {
    HAD_ERROR.store(b, Ordering::Relaxed);
}

fn had_runtime_error() -> bool {
    HAD_RUNTIME_ERROR.load(Ordering::Relaxed)
}

fn set_had_runtime_error(b: bool) {
    HAD_RUNTIME_ERROR.store(b, Ordering::Relaxed);
}
