use std::env;
use std::fs;
use venus_lexer::scanner::Scanner;
use venus_parser::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: venus_compiler <file.vs>");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            std::process::exit(1);
        }
    };

    // println!("Compiling {} ...", file_path);

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

    let mut analyzer = venus_analyzer::analyzer::SemanticAnalyzer::new(&source, file_path);
    if !analyzer.analyze(&ast) {
        std::process::exit(1);
    }

    let mut evaluator = venus_analyzer::eval::Evaluator::new();
    if let Err(e) = evaluator.eval_program(&ast) {
        let err = venus_analyzer::error::VenusError {
            title: "Runtime Error".to_string(),
            message: e,
            hint: None,
            span: evaluator.last_span.clone(),
        };
        let handler = venus_analyzer::error::VenusErrorHandler::new(&source, &args[1]);
        handler.report(&err);
        std::process::exit(1);
    }
}
