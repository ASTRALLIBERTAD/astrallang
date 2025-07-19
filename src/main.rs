mod compiler;

use compiler::lexer::tokenize;
use compiler::parser::Parser;
use compiler::interpreter::Interpreter;

fn main() {
    let code = std::fs::read_to_string("examples/hello.astral").expect("Failed to read source");

    let tokens = tokenize(&code);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    println!("{:#?}", ast);

    Interpreter::run(&ast);
}
