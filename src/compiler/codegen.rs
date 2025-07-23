// use crate::compiler::ast::*;
// use std::collections::HashMap;

// pub fn generate_x86(func: &Function) -> String {
//     let mut generator = X86Generator::new();
//     generator.generate_function(func)
// }

// struct X86Generator {
//     label_counter: usize,
//     variables: HashMap<String, i32>, // offset from rbp
//     stack_offset: i32,
// }

// impl X86Generator {
//     fn new() -> Self {
//         X86Generator {
//             label_counter: 0,
//             variables: HashMap::new(),
//             stack_offset: 0,
//         }
//     }

//     fn next_label(&mut self) -> String {
//         let label = format!("L{}", self.label_counter);
//         self.label_counter += 1;
//         label
//     }

//     fn generate_function(&mut self, func: &Function) -> String {
//         let mut asm = String::new();
        
//         // Assembly header
//         asm.push_str("section .data\n");
//         asm.push_str("    fmt db '%ld', 10, 0  ; format string for printf\n\n");
        
//         asm.push_str("section .text\n");
//         asm.push_str("global _start\n");
//         asm.push_str("extern printf\n");
//         asm.push_str("extern exit\n\n");
        
//         asm.push_str("_start:\n");
//         asm.push_str("    push rbp\n");
//         asm.push_str("    mov rbp, rsp\n");
        
//         // Generate function body
//         for stmt in &func.body {
//             asm.push_str(&self.generate_stmt(stmt));
//         }
        
//         // Function epilogue
//         asm.push_str("    mov rdi, 0\n");
//         asm.push_str("    call exit\n");
        
//         asm
//     }

//     fn generate_stmt(&mut self, stmt: &Stmt) -> String {
//         match stmt {
//             Stmt::Let(name, expr) => {
//                 let mut code = self.generate_expr(expr);
//                 self.stack_offset -= 8;
//                 self.variables.insert(name.clone(), self.stack_offset);
//                 code.push_str(&format!("    mov [rbp{}], rax\n", self.stack_offset));
//                 code
//             }
//             Stmt::Print(expr) => {
//                 let mut code = self.generate_expr(expr);
//                 code.push_str("    mov rsi, rax\n");
//                 code.push_str("    mov rdi, fmt\n");
//                 code.push_str("    mov rax, 0\n");
//                 code.push_str("    call printf\n");
//                 code
//             }
//             Stmt::If(condition, then_body, else_body) => {
//                 let mut code = self.generate_expr(condition);
//                 let else_label = self.next_label();
//                 let end_label = self.next_label();
                
//                 code.push_str("    cmp rax, 0\n");
//                 code.push_str(&format!("    je {}\n", else_label));
                
//                 for stmt in then_body {
//                     code.push_str(&self.generate_stmt(stmt));
//                 }
                
//                 code.push_str(&format!("    jmp {}\n", end_label));
//                 code.push_str(&format!("{}:\n", else_label));
                
//                 if let Some(else_stmts) = else_body {
//                     for stmt in else_stmts {
//                         code.push_str(&self.generate_stmt(stmt));
//                     }
//                 }
                
//                 code.push_str(&format!("{}:\n", end_label));
//                 code
//             }
//             Stmt::While(condition, body) => {
//                 let loop_label = self.next_label();
//                 let end_label = self.next_label();
//                 let mut code = String::new();
                
//                 code.push_str(&format!("{}:\n", loop_label));
//                 code.push_str(&self.generate_expr(condition));
//                 code.push_str("    cmp rax, 0\n");
//                 code.push_str(&format!("    je {}\n", end_label));
                
//                 for stmt in body {
//                     code.push_str(&self.generate_stmt(stmt));
//                 }
                
//                 code.push_str(&format!("    jmp {}\n", loop_label));
//                 code.push_str(&format!("{}:\n", end_label));
//                 code
//             }
//             Stmt::Expression(expr) => {
//                 self.generate_expr(expr)
//             }
//             _ => String::new(), // Other statements not implemented for codegen
//         }
//     }

//     fn generate_expr(&self, expr: &Expr) -> String {
//         match expr {
//             Expr::Number(n) => {
//                 format!("    mov rax, {}\n", n)
//             }
//             Expr::Ident(name) => {
//                 if let Some(&offset) = self.variables.get(name) {
//                     format!("    mov rax, [rbp{}]\n", offset)
//                 } else {
//                     format!("    mov rax, 0  ; undefined variable {}\n", name)
//                 }
//             }
//             Expr::BinaryOp(lhs, op, rhs) => {
//                 let mut code = self.generate_expr(lhs);
//                 code.push_str("    push rax\n");
//                 code.push_str(&self.generate_expr(rhs));
//                 code.push_str("    mov rbx, rax\n");
//                 code.push_str("    pop rax\n");
                
//                 match op {
//                     BinaryOperator::Add => code.push_str("    add rax, rbx\n"),
//                     BinaryOperator::Subtract => code.push_str("    sub rax, rbx\n"),
//                     BinaryOperator::Multiply => code.push_str("    imul rax, rbx\n"),
//                     BinaryOperator::Divide => {
//                         code.push_str("    cqo\n");
//                         code.push_str("    idiv rbx\n");
//                     }
//                     BinaryOperator::Equal => {
//                         code.push_str("    cmp rax, rbx\n");
//                         code.push_str("    sete al\n");
//                         code.push_str("    movzx rax, al\n");
//                     }
//                     BinaryOperator::Less => {
//                         code.push_str("    cmp rax, rbx\n");
//                         code.push_str("    setl al\n");
//                         code.push_str("    movzx rax, al\n");
//                     }
//                     BinaryOperator::Greater => {
//                         code.push_str("    cmp rax, rbx\n");
//                         code.push_str("    setg al\n");
//                         code.push_str("    movzx rax, al\n");
//                     }
//                     BinaryOperator::LessEqual => {
//                         code.push_str("    cmp rax, rbx\n");
//                         code.push_str("    setle al\n");
//                         code.push_str("    movzx rax, al\n");
//                     }
//                     BinaryOperator::GreaterEqual => {
//                         code.push_str("    cmp rax, rbx\n");
//                         code.push_str("    setge al\n");
//                         code.push_str("    movzx rax, al\n");
//                     }
//                     BinaryOperator::NotEqual => {
//                         code.push_str("    cmp rax, rbx\n");
//                         code.push_str("    setne al\n");
//                         code.push_str("    movzx rax, al\n");
//                     }
//                     BinaryOperator::Modulo => {
//                         code.push_str("    cqo\n");
//                         code.push_str("    idiv rbx\n");
//                         code.push_str("    mov rax, rdx\n");
//                     }
//                 }
                
//                 code
//             }
//             Expr::UnaryOp(op, expr) => {
//                 let mut code = self.generate_expr(expr);
//                 match op {
//                     UnaryOperator::Minus => code.push_str("    neg rax\n"),
//                     UnaryOperator::Not => {
//                         code.push_str("    cmp rax, 0\n");
//                         code.push_str("    sete al\n");
//                         code.push_str("    movzx rax, al\n");
//                     }
//                 }
//                 code
//             }
//             _ => String::new(), // Other expressions not implemented
//         }
//     }
// }