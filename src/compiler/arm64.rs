use crate::compiler::ast::*;
use std::collections::HashMap;

pub fn generate_arm64(func: &Function) -> String {
    let mut generator = ARM64Generator::new();
    generator.generate_function(func)
}

struct ARM64Generator {
    label_counter: usize,
    variables: HashMap<String, i32>, // offset from fp
    stack_offset: i32,
}

impl ARM64Generator {
    fn new() -> Self {
        ARM64Generator {
            label_counter: 0,
            variables: HashMap::new(),
            stack_offset: 0,
        }
    }

    fn next_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut asm = String::new();
        
        // ARM64 assembly header for Android
        asm.push_str(".section .text\n");
        asm.push_str(".global _start\n");

        asm.push_str(".global _start\n");
        
        // For Android, we need to use system calls directly
        asm.push_str("_start:\n");
        asm.push_str("    // Set up stack frame\n");
        asm.push_str("    stp x29, x30, [sp, #-16]!\n");
        asm.push_str("    mov x29, sp\n");


        // Exit system call for Android
        asm.push_str("    mov x8, #93      // sys_exit\n");
        asm.push_str("    mov x0, #0       // exit status\n");
        asm.push_str("    svc #0           // system call\n");

        // Generate function body
        for stmt in &func.body {
            asm.push_str(&self.generate_stmt(stmt));
        }
        
        // Exit system call for Android
        // Exit system call for Android
        asm.push_str("    mov x8, #93      // sys_exit\n");
        asm.push_str("    mov x0, #0       // exit status\n");
        asm.push_str("    svc #0           // system call\n");

        

        // Add data section for print buffer
        asm.push_str("\n.section .data\n");
        asm.push_str("buf: .space 20\n");

        asm
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Let(name, expr) => {
                let mut code = self.generate_expr(expr);
                self.stack_offset -= 8;
                self.variables.insert(name.clone(), self.stack_offset);
                code.push_str(&format!("    str x0, [x29, #{}]\n", self.stack_offset));
                code
            }
            Stmt::Print(expr) => {
                let mut code = String::new();

                // 1. Evaluate the expression — result in x0
                code.push_str(&self.generate_expr(expr)); // result in x0

                // 2. Setup pointer to buffer (buf + 19), PC-relative addressing
                code.push_str("    adrp x1, buf\n");
                code.push_str("    add x1, x1, :lo12:buf\n");
                code.push_str("    add x1, x1, #19\n");
                code.push_str("    mov x2, #0\n"); // digit counter

                // 3. Convert number in x0 to ASCII (base 10)
                code.push_str("convert_loop:\n");
                code.push_str("    mov x3, #10\n");
                code.push_str("    udiv x4, x0, x3\n"); // x4 = x0 / 10
                code.push_str("    msub x5, x4, x3, x0\n"); // x5 = x0 - (x4 * 10) = remainder
                code.push_str("    add x5, x5, #'0'\n"); // convert to ASCII
                code.push_str("    strb w5, [x1], #-1\n"); // store byte, move back
                code.push_str("    mov x0, x4\n");
                code.push_str("    add x2, x2, #1\n"); // increment digit count
                code.push_str("    cmp x0, #0\n");
                code.push_str("    b.ne convert_loop\n");

                // 4. sys_write: write to stdout
                code.push_str("    mov x0, #1\n");      // stdout
                code.push_str("    add x1, x1, #1\n");  // point to start of ASCII
                code.push_str("    mov x2, x2\n");      // length
                code.push_str("    mov x8, #64\n");     // syscall: write
                code.push_str("    svc #0\n");

                code
            }


            Stmt::If(condition, then_body, else_body) => {
                let mut code = self.generate_expr(condition);
                let else_label = self.next_label();
                let end_label = self.next_label();
                
                code.push_str("    cmp x0, #0\n");
                code.push_str(&format!("    b.eq {}\n", else_label));
                
                for stmt in then_body {
                    code.push_str(&self.generate_stmt(stmt));
                }
                
                code.push_str(&format!("    b {}\n", end_label));
                code.push_str(&format!("{}:\n", else_label));
                
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        code.push_str(&self.generate_stmt(stmt));
                    }
                }
                
                code.push_str(&format!("{}:\n", end_label));
                code
            }
            Stmt::While(condition, body) => {
                let loop_label = self.next_label();
                let end_label = self.next_label();
                let mut code = String::new();
                
                code.push_str(&format!("{}:\n", loop_label));
                code.push_str(&self.generate_expr(condition));
                code.push_str("    cmp x0, #0\n");
                code.push_str(&format!("    b.eq {}\n", end_label));
                
                for stmt in body {
                    code.push_str(&self.generate_stmt(stmt));
                }
                
                code.push_str(&format!("    b {}\n", loop_label));
                code.push_str(&format!("{}:\n", end_label));
                code
            }
            Stmt::Expression(expr) => {
                self.generate_expr(expr)
            }
            _ => String::new(),
        }
    }

    fn generate_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => {
                format!("    mov x0, #{}\n", n)
            }
            Expr::Ident(name) => {
                if let Some(&offset) = self.variables.get(name) {
                    format!("    ldr x0, [x29, #{}]\n", offset)
                } else {
                    format!("    mov x0, #0  // undefined variable {}\n", name)
                }
            }
            Expr::BinaryOp(lhs, op, rhs) => {
                let mut code = self.generate_expr(lhs);
                code.push_str("    str x0, [sp, #-16]!\n");  // push to stack
                code.push_str(&self.generate_expr(rhs));
                code.push_str("    mov x1, x0\n");
                code.push_str("    ldr x0, [sp], #16\n");   // pop from stack
                
                match op {
                    BinaryOperator::Add => code.push_str("    add x0, x0, x1\n"),
                    BinaryOperator::Subtract => code.push_str("    sub x0, x0, x1\n"),
                    BinaryOperator::Multiply => code.push_str("    mul x0, x0, x1\n"),
                    BinaryOperator::Divide => {
                        code.push_str("    sdiv x0, x0, x1\n");
                    }
                    BinaryOperator::Equal => {
                        code.push_str("    cmp x0, x1\n");
                        code.push_str("    cset x0, eq\n");
                    }
                    BinaryOperator::Less => {
                        code.push_str("    cmp x0, x1\n");
                        code.push_str("    cset x0, lt\n");
                    }
                    BinaryOperator::Greater => {
                        code.push_str("    cmp x0, x1\n");
                        code.push_str("    cset x0, gt\n");
                    }
                    BinaryOperator::LessEqual => {
                        code.push_str("    cmp x0, x1\n");
                        code.push_str("    cset x0, le\n");
                    }
                    BinaryOperator::GreaterEqual => {
                        code.push_str("    cmp x0, x1\n");
                        code.push_str("    cset x0, ge\n");
                    }
                    BinaryOperator::NotEqual => {
                        code.push_str("    cmp x0, x1\n");
                        code.push_str("    cset x0, ne\n");
                    }
                    BinaryOperator::Modulo => {
                        code.push_str("    sdiv x2, x0, x1\n");
                        code.push_str("    mul x2, x2, x1\n");
                        code.push_str("    sub x0, x0, x2\n");
                    }
                }
                
                code
            }
            Expr::UnaryOp(op, expr) => {
                let mut code = self.generate_expr(expr);
                match op {
                    UnaryOperator::Minus => code.push_str("    neg x0, x0\n"),
                    UnaryOperator::Not => {
                        code.push_str("    cmp x0, #0\n");
                        code.push_str("    cset x0, eq\n");
                    }
                }
                code
            }
            _ => String::new(),
        }
    }
}