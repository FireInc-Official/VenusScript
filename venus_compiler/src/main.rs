pub mod lexer;
pub mod parser;
pub mod analyzer;

use std::env;
use std::fs;
use crate::lexer::scanner::Scanner;
use crate::parser::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let first_arg = &args[1];
    match first_arg.as_str() {
        "--version" | "-v" => {
            println!("VenusScript Compiler v0.2.0");
            std::process::exit(0);
        }
        "--help" | "-h" => {
            print_usage();
            std::process::exit(0);
        }
        _ => {}
    }

    let file_path = first_arg;

    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            std::process::exit(1);
        }
    };

    let mut scanner = Scanner::new(&source);
    let tokens = match scanner.scan_all() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parser Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut analyzer = crate::analyzer::analyzer::SemanticAnalyzer::new(&source, file_path);
    if !analyzer.analyze(&ast) {
        std::process::exit(1);
    }

    let mut evaluator = crate::analyzer::eval::Evaluator::new();
    if let Err(e) = evaluator.eval_program(&ast) {
        let err = crate::analyzer::error::VenusError {
            title: "Runtime Error".to_string(),
            message: e,
            hint: None,
            span: evaluator.last_span.clone(),
        };
        let handler = crate::analyzer::error::VenusErrorHandler::new(&source, &args[1]);
        handler.report(&err);
        std::process::exit(1);
    }
}

fn print_usage() {
    println!("VenusScript Compiler v0.2.0");
    println!("Usage:");
    println!("  vscript <file.vs>   Compile and run a VenusScript file");
    println!("  vscript --version   Print compiler version");
    println!("  vscript --help      Print this help message");
}
