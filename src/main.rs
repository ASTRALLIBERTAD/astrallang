mod compiler;

use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::interpreter::Interpreter;
use compiler::codegen::generate_x86;
use std::process;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_file> [--compile]", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let compile_mode = args.len() > 2 && args[2] == "--compile";

    let code = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    // Tokenize
    let mut lexer = Lexer::new(&code);
    let tokens = match Lexer::tokenize(&mut lexer) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // Debug: Print AST if requested
    if std::env::var("DEBUG_AST").is_ok() {
        println!("AST: {:#?}", ast);
    }

    if compile_mode {
        // Compile to assembly
        if let Some(main_func) = ast.iter().find_map(|stmt| {
            if let compiler::ast::Stmt::Function(func) = stmt {
                if func.name == "main" {
                    Some(func)
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            let asm_code = generate_x86(main_func);
            let output_file = filename.replace(".astral", ".asm");
            if let Err(e) = fs::write(&output_file, asm_code) {
                eprintln!("Failed to write assembly file '{}': {}", output_file, e);
                process::exit(1);
            }
            println!("Assembly code written to: {}", output_file);
        } else {
            eprintln!("No main function found for compilation");
            process::exit(1);
        }
    } else {
        // Interpret
        let mut interpreter = Interpreter::new();
        if let Err(e) = interpreter.run(&ast) {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}