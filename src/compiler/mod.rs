pub mod lexer;
pub mod parser;
pub mod ast;
pub mod interpreter;
pub mod semantic;
// pub mod llvm;
pub mod llvm_android;
pub mod value;
pub mod android_compiler;
pub mod builtins;

pub use ast::*;
pub use lexer::Lexer;
pub use parser::Parser;
pub use interpreter::Interpreter;

// // Android compilation functions
// pub use codegen_inkwell::{
//     InkwellAndroidCodegen,
//     generate_llvm_android_inkwell,
//     compile_to_llvm_android_inkwell,
//     compile_to_android_object,
// };

// pub use android_compiler::AndroidCompiler;