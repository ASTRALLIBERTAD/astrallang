mod compiler;

use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::interpreter::Interpreter;
use compiler::codegen::generate_x86;
use compiler::arm64::generate_arm64;
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
            // Optional: emit ARM64 or x86 assembly
            if args.contains(&"--android".to_string()) {
                let asm_code = generate_arm64(main_func);
                let output_file = filename.replace(".astral", ".s");
                if let Err(e) = fs::write(&output_file, asm_code) {
                    eprintln!("Failed to write ARM64 assembly: {}", e);
                    process::exit(1);
                }
                println!("ARM64 assembly written to: {}", output_file);
            } else {
                let asm_code = generate_x86(main_func);
                let output_file = filename.replace(".astral", ".asm");
                if let Err(e) = fs::write(&output_file, asm_code) {
                    eprintln!("Failed to write x86 assembly: {}", e);
                    process::exit(1);
                }
                println!("x86 assembly written to: {}", output_file);
            }

            // 🔥 Now ALSO emit LLVM IR if requested
            if args.contains(&"--llvm".to_string()) {
                use compiler::llvm::generate_llvm;

                let llvm_ir = generate_llvm(main_func);
                let llvm_file = filename.replace(".astral", ".ll");
                if let Err(e) = fs::write(&llvm_file, llvm_ir) {
                    eprintln!("Failed to write LLVM IR file '{}': {}", llvm_file, e);
                    process::exit(1);
                }
                println!("LLVM IR written to: {}", llvm_file);
            }

            if args.contains(&"--interpreter".to_string()) {
                let code = std::fs::read_to_string("examples/hello.astral").expect("Failed to read source");

                let mut lexer = Lexer::new(&code);
                let tokens = match Lexer::tokenize(&mut lexer) {
                    Ok(tokens) => tokens,
                    Err(e) => {
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };
                let mut parser = Parser::new(tokens);
                let ast = match parser.parse() {
                    Ok(ast) => ast,
                    Err(e) => {
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };

                
                println!("{:#?}", ast);
                let mut interpreter = Interpreter::new();
                if let Err(e) = interpreter.run(&ast) {
                    eprintln!("Interpreter error: {}", e);
                    process::exit(1);
                }
            }
        } else {
            eprintln!("No main function found for compilation");
            process::exit(1);
        }
    }
}