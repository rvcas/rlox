use std::env;

use rlox::lox;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
    } else if args.len() == 2 {
        lox::run_file(args[1].as_str());
    } else {
        lox::run_prompt();
    }
}
