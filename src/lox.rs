use std::{
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{parser::Parser, scanner::Scanner, token::Token, token_type::TokenType};

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

pub fn run_file(path_name: &str) {
    let file_path = Path::new(path_name);

    let file_res = File::open(file_path);

    match file_res {
        Ok(mut src_file) => {
            let mut src = String::new();

            let read_res = src_file.read_to_string(&mut src);

            match read_res {
                Ok(_) => run(&src),
                Err(_) => println!("error: could not read {}", path_name),
            }
        }
        Err(_) => println!("error: could not open {}", path_name),
    };
}

pub fn run_prompt() {
    let mut input = String::new();

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

                run(&input);

                set_had_error(false);
            }
            Err(_) => {
                println!("error: bad input");
            }
        }

        input.clear();
    }
}

fn run(src: &str) {
    if had_error() {
        std::process::exit(65);
    }

    let mut scanner = Scanner::new(src);

    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens.clone());

    let ast = parser.parse();

    println!("{:?}", ast.unwrap());
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, where_: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, where_, message);

    set_had_error(true);
}

pub fn parse_error(token: Token, message: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, " at end", message)
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme), message)
    }
}

fn had_error() -> bool {
    HAD_ERROR.load(Ordering::Relaxed)
}

fn set_had_error(b: bool) {
    HAD_ERROR.store(b, Ordering::Relaxed);
}
