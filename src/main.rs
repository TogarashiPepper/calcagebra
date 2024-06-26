mod ast;
mod data;
mod interpreter;
mod lexer;
mod parser;
mod standardlibrary;
mod token;

use rustyline::{error::ReadlineError, DefaultEditor};
use std::{fs::read_to_string, process::exit, time::Instant};

use clap::Parser as ClapParser;
use lexer::Lexer;

use crate::{interpreter::Interpreter, parser::Parser};

#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Output debug information
    #[clap(short, long, value_parser)]
    debug: bool,

    /// Only run the lexer and parser
    #[clap(long, value_parser)]
    dry_run: bool,

    /// Print the time elapsed while executing code
    #[clap(short, long, value_parser)]
    time: bool,

    /// Print the characters used as globals
    #[clap(short, long, value_parser)]
    globals: bool,

    /// The path of file which is to be executed
    #[clap()]
    input: Option<String>,
}

fn main() {
    let args = Args::parse();
    let main = Instant::now();

    if args.input.is_none() {
        if args.globals {
            println!("calcagebra v{}\n", version());
            let _ = Interpreter::new()
                .init_globals()
                .variables
                .iter()
                .map(|(a, b)| println!("{a} {b}"))
                .collect::<Vec<_>>();
            return;
        }
        println!(
            "Welcome to calcagebra v{}\nTo exit, press CTRL+C or CTRL+D",
            version()
        );
        let mut rl = DefaultEditor::new().unwrap();
        let mut interpreter = Interpreter::new();
        loop {
            let readline = rl.readline("\x1b[1m\x1b[32m[In]:\x1b[0m ");
            match readline {
                Ok(line) => {
                    if line.trim() == "exit" {
                        break;
                    }

                    println!("\x1b[1m\x1b[31m[Out]:\x1b[0m ");

                    interpreter.run(Parser::new(Lexer::new(&line).tokens()).ast());
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        exit(1);
    }

    let contents = read_to_string(args.input.unwrap()).unwrap();

    let tokens = Lexer::new(&contents).tokens();

    if args.debug {
        let duration = main.elapsed();
        println!("LEXER: {tokens:?}\n\nTIME: {duration:?}\n");
    }

    let ast = Parser::new(tokens).ast();

    if args.debug {
        let duration = main.elapsed();
        println!("AST: {ast:?}\n\nTIME: {duration:?}\n");
    }

    if args.dry_run {
        return;
    }

    Interpreter::new().run(ast);

    if args.debug || args.time {
        let duration = main.elapsed();
        println!("\nTIME: {duration:?}");
    }
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
