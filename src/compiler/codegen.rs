use crate::compiler::ast::*;

pub fn generate_x86(func: &Function) -> String {
    let mut asm = String::new();
    asm.push_str("section .text\n");
    asm.push_str("global _start\n");
    asm.push_str("_start:\n");

    for stmt in &func.body {
        match stmt {
            Stmt::Let(_, Expr::Number(n)) => {
                asm.push_str(&format!("    mov rax, {}\n", n));
            }
            Stmt::Print(Expr::Number(n)) => {
                asm.push_str(&format!("    mov rdi, {}\n", n));
                asm.push_str("    call print_number\n");
            }
            _ => {}
        }
    }

    asm.push_str("    hlt\n");
    asm.push_str("print_number:\n");
    asm.push_str("    ret\n");

    asm
}
