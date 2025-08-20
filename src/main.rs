mod compiler;

use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::interpreter::Interpreter;
// use compiler::android_compiler::AndroidCompiler;
use std::fs;
use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename.astral> [--android] [--object] [--run-android] [--llvm] [--interpreter] [--compile]", args[0]);
        eprintln!("Options:");
        eprintln!("  --android     Generate LLVM IR for Android target");
        eprintln!("  --object      Compile directly to Android object file");
        eprintln!("  --run-android Compile and run on connected Android device");
        eprintln!("  --llvm        Generate LLVM IR for desktop");
        eprintln!("  --interpreter Run using interpreter");
        process::exit(1);
    }

    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer Error: {}", e);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parser Error: {}", e);
            process::exit(1);
        }
    };

    // if args.contains(&"--llvm".to_string()) {
    //     use compiler::llvm::compile_to_llvm;
    //     let llvm_ir = match compile_to_llvm(&ast) {
    //         Ok(ir) => ir,
    //         Err(e) => {
    //             eprintln!("LLVM Codegen Error: {}", e);
    //             process::exit(1);
    //         }
    //     };
    //     let llvm_file = filename.replace(".astral", ".ll");
    //     if let Err(e) = fs::write(&llvm_file, llvm_ir) {
    //         eprintln!("Failed to write LLVM IR file '{}': {}", llvm_file, e);
    //         process::exit(1);
    //     }
    //     println!("LLVM IR written to: {}", llvm_file);
    // }

    // if args.contains(&"--android".to_string()) {
    //     use compiler::llvm_android::compile_to_llvm_android;
    //     let llvm_ir = match compile_to_llvm_android(&ast) {
    //         Ok(ir) => ir,
    //         Err(e) => {
    //             eprintln!("LLVM Codegen Error: {}", e);
    //             process::exit(1);
    //         }
    //     };
    //     let llvm_file = filename.replace(".astral", "_android.ll");
    //     if let Err(e) = fs::write(&llvm_file, llvm_ir) {
    //         eprintln!("Failed to write LLVM IR file '{}': {}", llvm_file, e);
    //         process::exit(1);
    //     }
    //     println!("✅ Android LLVM IR written to: {}", llvm_file);
    // }

    // if args.contains(&"--run-android".to_string()) {
    //     let ndk_path = match env::var("ANDROID_NDK_ROOT") {
    //         Ok(p) => p,
    //         Err(_) => {
    //             eprintln!("ANDROID_NDK_ROOT is not set");
    //             process::exit(1);
    //         }
    //     };
    //     let compiler = AndroidCompiler::new(ndk_path, 35);
    //     match compiler.compile_and_run_on_device(&source, None) {
    //         Ok(output) => println!("{}", output),
    //         Err(e) => {
    //             eprintln!("Failed to run on Android: {}", e);
    //             process::exit(1);
    //         }
    //     }
    // }

    if args.contains(&"--interpreter".to_string()) || (!args.contains(&"--compile".to_string()) && !args.contains(&"--llvm".to_string()) && !args.contains(&"--android".to_string())) {
        let mut interpreter = Interpreter::new();
        print!("{:#?}", ast);
        if let Err(e) = interpreter.run(&ast) {
            eprintln!("Runtime Error: {}", e);
            process::exit(1);
        }
    }

    if args.contains(&"--compile".to_string()) {
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
            println!("Main function found. Compilation placeholder executed.");
        } else {
            eprintln!("No main function found for compilation");
            process::exit(1);
        }
    }
}
